#pragma once
#include <atomic>

#ifdef __APPLE__
#include <OpenAL/alc.h>
#else
#include <AL/alc.h>
#endif

// The embedding owner creates an original OpenAL context and retains it until
// all native wave buffers/workers and this renderer wrapper have been released.
struct TVPEmbeddedAudioContext {
    ALCdevice *device;
    ALCcontext *context;
};
inline std::atomic<const TVPEmbeddedAudioContext *> TVPEmbeddedAudioContextSlot{nullptr};

inline bool TVPInstallEmbeddedAudioContext(const TVPEmbeddedAudioContext *context) noexcept {
    if(!context || !context->device || !context->context)
        return false;
    const TVPEmbeddedAudioContext *expected = nullptr;
    return TVPEmbeddedAudioContextSlot.compare_exchange_strong(
        expected, context, std::memory_order_release, std::memory_order_relaxed);
}
inline const TVPEmbeddedAudioContext *TVPGetEmbeddedAudioContext() noexcept {
    return TVPEmbeddedAudioContextSlot.load(std::memory_order_acquire);
}
inline bool TVPRemoveEmbeddedAudioContext(const TVPEmbeddedAudioContext *context) noexcept {
    auto expected = context;
    return TVPEmbeddedAudioContextSlot.compare_exchange_strong(
        expected, nullptr, std::memory_order_acq_rel, std::memory_order_acquire);
}

// Called only after the original VM/wave shutdown, never to replace a live mixer.
void TVPReleaseEmbeddedAudioRenderer();
