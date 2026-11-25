#include "AndroidHost.h"
#include "include/shinki_emu.h"
#include "platform/android/jni/JniHelper.h"
#include "environ/EmbeddedSession.h"
#include "Platform.h"
#include "environ/cocos2d/MainScene.h"
#include <jni.h>
#include <memory>
#include <stdexcept>
#include <string>
#include <atomic>
#include <mutex>

extern "C" {
// This name is applied to SDL's original loader before closure resolution.
// Cocos' Activity loader is never forced into the embedded link.
jint shinki_krkr_sdl_on_load(JavaVM *, void *);
void Java_org_cocos2dx_lib_Cocos2dxHelper_nativeSetContext(JNIEnv *, jobject, jobject, jobject);
void Java_org_cocos2dx_lib_Cocos2dxHelper_nativeSetAudioDeviceInfo(JNIEnv *, jobject, jboolean, jint, jint);
void Java_org_cocos2dx_lib_Cocos2dxHelper_nativeSetEditTextDialogResult(JNIEnv *, jobject, jbyteArray);
void Java_org_cocos2dx_lib_Cocos2dxBitmap_nativeInitBitmapDC(JNIEnv *, jobject, jint, jint, jbyteArray);
void Java_org_cocos2dx_lib_Cocos2dxAccelerometer_onSensorChanged(JNIEnv *, jobject, jfloat, jfloat, jfloat, jlong);
}

namespace {
JavaVM *host_vm = nullptr;
jclass host_class = nullptr;
jmethodID start_method = nullptr, drain_method = nullptr, stop_method = nullptr;
jmethodID initialize_method = nullptr;
#ifdef SHINKI_EMU_STATIC_CORE
std::atomic<bool> ready{false};
#else
jobject host_application = nullptr; // Borrowed process-lifetime JNI global.
bool registration_attempted = false;
#endif
std::mutex initialize_mutex;
class LocalFrame {
    JNIEnv *env_;
public:
    explicit LocalFrame(JNIEnv *env) : env_(env) {
        if(env_->PushLocalFrame(32) != JNI_OK) throw std::runtime_error("KRKR_JNI_LOCAL_FRAME");
    }
    ~LocalFrame() { env_->PopLocalFrame(nullptr); }
};

void check(JNIEnv *env, const char *code) {
    if(env->ExceptionCheck()) {
        env->ExceptionDescribe();
        env->ExceptionClear();
        throw std::runtime_error(code);
    }
}
jint initialize_sdl(JNIEnv *env, jclass) {
    const auto version = shinki_krkr_sdl_on_load(host_vm, nullptr);
    return env->ExceptionCheck() ? JNI_ERR : version;
}
void throw_java(JNIEnv *env, const char *code) noexcept {
    if(env->ExceptionCheck()) return;
    const auto type = env->FindClass("java/lang/IllegalStateException");
    if(type) { env->ThrowNew(type, code); env->DeleteLocalRef(type); }
}
void request_exit(JNIEnv *env, jclass) {
    if(const auto host = TVPGetEmbeddedSessionHost())
        host->request_exit(host->context, 0);
    else throw_java(env, "KRKR_OWNER_NOT_RUNNING");
}
void commit_text(JNIEnv *env, jclass, jstring value) {
    auto bytes = env->GetStringUTFChars(value, nullptr);
    if(!bytes) return; // JNI's pending allocation exception reaches Java.
    try {
        const std::string text(bytes);
        env->ReleaseStringUTFChars(value, bytes);
        bytes = nullptr;
        TVPMainScene::onTextInput(text);
    } catch(...) {
        if(bytes) env->ReleaseStringUTFChars(value, bytes);
        throw_java(env, "KRKR_TEXT_INPUT_FAILED");
    }
}
void register_methods(JNIEnv *env, const char *name, JNINativeMethod *methods, jint count) {
    // JniHelper uses the application classloader even on attached owner threads.
    cocos2d::JniMethodInfo lookup;
    if(!cocos2d::JniHelper::getStaticMethodInfo(lookup, name, methods[0].name, methods[0].signature))
        throw std::runtime_error("KRKR_JNI_CLASS_OR_SIGNATURE");
    const auto result = env->RegisterNatives(lookup.classID, methods, count);
    env->DeleteLocalRef(lookup.classID);
    check(env, "KRKR_JNI_REGISTER");
    if(result != JNI_OK) throw std::runtime_error("KRKR_JNI_REGISTER");
}
void owner_call(jmethodID method) {
    // Original JniHelper attaches native-created threads and registers its TLS
    // destructor to DetachCurrentThread. No JNIEnv crosses a thread boundary.
    auto env = cocos2d::JniHelper::getEnv();
    if(!env || !host_class || !method) throw std::runtime_error("KRKR_ANDROID_HOST_UNINITIALIZED");
    LocalFrame frame(env);
    env->CallStaticVoidMethod(host_class, method);
    check(env, "KRKR_JAVA_OWNER_CALLBACK");
}

void register_host(JNIEnv *env) {
    JNINativeMethod helper[] = {
        {const_cast<char *>("nativeSetContext"), const_cast<char *>("(Landroid/content/Context;Landroid/content/res/AssetManager;)V"), reinterpret_cast<void *>(Java_org_cocos2dx_lib_Cocos2dxHelper_nativeSetContext)},
        {const_cast<char *>("nativeSetAudioDeviceInfo"), const_cast<char *>("(ZII)V"), reinterpret_cast<void *>(Java_org_cocos2dx_lib_Cocos2dxHelper_nativeSetAudioDeviceInfo)},
        {const_cast<char *>("nativeSetEditTextDialogResult"), const_cast<char *>("([B)V"), reinterpret_cast<void *>(Java_org_cocos2dx_lib_Cocos2dxHelper_nativeSetEditTextDialogResult)},
    };
    JNINativeMethod bitmap[] = {{const_cast<char *>("nativeInitBitmapDC"), const_cast<char *>("(II[B)V"), reinterpret_cast<void *>(Java_org_cocos2dx_lib_Cocos2dxBitmap_nativeInitBitmapDC)}};
    JNINativeMethod sensor[] = {{const_cast<char *>("onSensorChanged"), const_cast<char *>("(FFFJ)V"), reinterpret_cast<void *>(Java_org_cocos2dx_lib_Cocos2dxAccelerometer_onSensorChanged)}};
    JNINativeMethod platform[] = {{const_cast<char *>("nativeCommitText"), const_cast<char *>("(Ljava/lang/String;)V"), reinterpret_cast<void *>(commit_text)}};
    JNINativeMethod host[] = {
        {const_cast<char *>("nativeInitialize"), const_cast<char *>("()I"), reinterpret_cast<void *>(initialize_sdl)},
        {const_cast<char *>("nativeRequestExit"), const_cast<char *>("()V"), reinterpret_cast<void *>(request_exit)},
    };
    register_methods(env, "org.cocos2dx.lib.Cocos2dxHelper", helper, 3);
    register_methods(env, "org.cocos2dx.lib.Cocos2dxBitmap", bitmap, 1);
    register_methods(env, "org.cocos2dx.lib.Cocos2dxAccelerometer", sensor, 1);
    register_methods(env, "org.tvp.kirikiri2.KR2Activity", platform, 1);
    register_methods(env, "org.shinkirou.krkr.KrkrHost", host, 2);
    cocos2d::JniMethodInfo initialize;
#ifdef SHINKI_EMU_STATIC_CORE
    const bool found = cocos2d::JniHelper::getStaticMethodInfo(initialize,
        "org.shinkirou.krkr.KrkrHost", "initialize", "(Landroid/app/Activity;)V");
#else
    const bool found = cocos2d::JniHelper::getStaticMethodInfo(initialize,
        "org.shinkirou.krkr.KrkrHost", "initializeForActivation", "()V");
#endif
    if(!found) throw std::runtime_error("KRKR_JAVA_HOST_INITIALIZER");
    initialize_method = initialize.methodID;
    start_method = env->GetStaticMethodID(initialize.classID, "ownerStarted", "()V");
    drain_method = env->GetStaticMethodID(initialize.classID, "drain", "()V");
    stop_method = env->GetStaticMethodID(initialize.classID, "ownerStopped", "()V");
    check(env, "KRKR_JAVA_OWNER_SIGNATURES");
    if(!start_method || !drain_method || !stop_method) throw std::runtime_error("KRKR_JAVA_OWNER_SIGNATURES");
    host_class = static_cast<jclass>(env->NewGlobalRef(initialize.classID));
    check(env, "KRKR_JAVA_HOST_CLASS");
    if(!host_class) throw std::runtime_error("KRKR_JAVA_HOST_CLASS");
}
}

#ifdef SHINKI_EMU_STATIC_CORE
extern "C" ShinkiEmuStatus shinki_emu_kirikiroid2_web_android_initialize(void *vm, void *activity) noexcept {
    try {
        std::lock_guard<std::mutex> lock(initialize_mutex);
        if(!vm || !activity || (host_vm && host_vm != vm)) return ShinkiEmuINVALID_STATE;
        auto java_vm = static_cast<JavaVM *>(vm);
        JNIEnv *env = nullptr;
        if(java_vm->GetEnv(reinterpret_cast<void **>(&env), JNI_VERSION_1_6) != JNI_OK) return ShinkiEmuINVALID_STATE;
        LocalFrame frame(env);
        if(!host_class) {
            if(!host_vm) {
                host_vm = java_vm;
                cocos2d::JniHelper::setJavaVM(host_vm);
            }
            auto activity_class = env->GetObjectClass(static_cast<jobject>(activity));
            check(env, "KRKR_ACTIVITY_CLASS");
            if(!activity_class) throw std::runtime_error("KRKR_ACTIVITY_CLASS");
            auto get_context = env->GetMethodID(activity_class, "getApplicationContext", "()Landroid/content/Context;");
            check(env, "KRKR_APPLICATION_CONTEXT");
            if(!get_context) throw std::runtime_error("KRKR_APPLICATION_CONTEXT");
            auto context = env->CallObjectMethod(static_cast<jobject>(activity), get_context);
            check(env, "KRKR_APPLICATION_CONTEXT");
            // Keep the application classloader when NativeActivity is recreated.
            if(!context || !cocos2d::JniHelper::setClassLoaderFrom(context)) throw std::runtime_error("KRKR_CLASSLOADER");
            env->DeleteLocalRef(context);
            env->DeleteLocalRef(activity_class);
            register_host(env);
        }
        ready.store(false, std::memory_order_release);
        env->CallStaticVoidMethod(host_class, initialize_method, static_cast<jobject>(activity));
        check(env, "KRKR_JAVA_HOST_INITIALIZATION");
        ready.store(true, std::memory_order_release);
        return ShinkiEmuOK;
    } catch(...) {
        return ShinkiEmuCORE_ERROR;
    }
}
#else
bool shinki::krkr::android::set_context(void *vm, void *application) {
    std::lock_guard<std::mutex> lock(initialize_mutex);
    if(!vm || !application) return false;
    if(host_vm) return host_vm == vm && host_application == application;
    host_vm = static_cast<JavaVM *>(vm);
    host_application = static_cast<jobject>(application);
    return true;
}

std::filesystem::path shinki::krkr::android::activate() {
    std::lock_guard<std::mutex> lock(initialize_mutex);
    if(!host_application) throw std::runtime_error("KRKR_ANDROID_CONTEXT_REQUIRED");
    if(!host_class && registration_attempted) throw std::runtime_error("KRKR_JNI_REGISTRATION_FAILED");
    if(!registration_attempted) {
        registration_attempted = true;
        cocos2d::JniHelper::setJavaVM(host_vm);
    }
    auto env = cocos2d::JniHelper::getEnv();
    if(!env) throw std::runtime_error("KRKR_ANDROID_JNI_ATTACH");
    LocalFrame frame(env);
    if(!host_class) {
        const bool classloader = cocos2d::JniHelper::setClassLoaderFrom(host_application);
        check(env, "KRKR_CLASSLOADER");
        if(!classloader) throw std::runtime_error("KRKR_CLASSLOADER");
        register_host(env);
    }
    env->CallStaticVoidMethod(host_class, initialize_method);
    check(env, "KRKR_JAVA_HOST_INITIALIZATION");

    // Keep scratch out of the installed package and game directories.
    auto context_class = env->GetObjectClass(host_application);
    check(env, "KRKR_APPLICATION_CONTEXT");
    auto get_cache = env->GetMethodID(context_class, "getCacheDir", "()Ljava/io/File;");
    check(env, "KRKR_ANDROID_CACHE_DIR");
    auto cache = env->CallObjectMethod(host_application, get_cache);
    check(env, "KRKR_ANDROID_CACHE_DIR");
    if(!cache) throw std::runtime_error("KRKR_ANDROID_CACHE_DIR");
    auto file_class = env->GetObjectClass(cache);
    check(env, "KRKR_ANDROID_CACHE_DIR");
    auto get_absolute = env->GetMethodID(file_class, "getAbsolutePath", "()Ljava/lang/String;");
    check(env, "KRKR_ANDROID_CACHE_DIR");
    auto path = static_cast<jstring>(env->CallObjectMethod(cache, get_absolute));
    check(env, "KRKR_ANDROID_CACHE_DIR");
    if(!path) throw std::runtime_error("KRKR_ANDROID_CACHE_DIR");
    auto release = [=](const char *chars) { env->ReleaseStringUTFChars(path, chars); };
    const std::unique_ptr<const char, decltype(release)> chars(env->GetStringUTFChars(path, nullptr), release);
    check(env, "KRKR_ANDROID_CACHE_DIR");
    if(!chars) throw std::runtime_error("KRKR_ANDROID_CACHE_DIR");
    auto directory = std::filesystem::u8path(chars.get());
    if(!directory.is_absolute()) throw std::runtime_error("KRKR_ANDROID_CACHE_DIR");
    return directory;
}
#endif

namespace shinki::krkr::android {
#ifdef SHINKI_EMU_STATIC_CORE
bool initialized() noexcept { return ready.load(std::memory_order_acquire); }
#endif
void owner_started() { owner_call(start_method); }
void drain() { owner_call(drain_method); }
void owner_stopped() { owner_call(stop_method); }
}
