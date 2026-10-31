use shinki_emu_build::{
    configure_cmake, write_core_build_manifest, BuildAsset, CoreBuildManifest,
};
use std::{env, fs, path::PathBuf};
mod android_static;

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let target = env::var("TARGET")?;
    let host = env::var("HOST")?;
    let supported = [
        "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "x86_64-apple-darwin",
        "aarch64-apple-darwin", "aarch64-apple-ios", "aarch64-apple-ios-sim",
        "x86_64-apple-ios", "aarch64-linux-android", "x86_64-linux-android",
    ];
    if !supported.contains(&target.as_str()) {
        return Err(format!("KiriKiri unsupported TARGET={target}").into());
    }
    if target == "x86_64-pc-windows-msvc" && host != target {
        return Err(format!("KiriKiri Windows build requires native HOST={target}; HOST={host}").into());
    }
    for name in ["PATH", "NUM_JOBS", "VCToolsInstallDir", "WindowsSdkDir"] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    let adapter = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or("missing CARGO_MANIFEST_DIR")?);
    let root = adapter.parent().ok_or("adapter has no parent")?;
    let vcpkg = PathBuf::from(env::var_os("VCPKG_ROOT").ok_or(
        "VCPKG_ROOT is missing; run cargo xtask deps for kirikiroid2-web",
    )?);
    let toolchain = vcpkg.join("scripts/buildsystems/vcpkg.cmake");
    let triplet = env::var("SHINKI_EMU_VCPKG_TRIPLET")?;
    let installed = PathBuf::from(env::var_os("SHINKI_EMU_NATIVE_VCPKG_ROOT").ok_or(
        "SHINKI_EMU_NATIVE_VCPKG_ROOT is missing; run cargo xtask deps",
    )?).join("kirikiroid2-web").join(format!("installed-{triplet}"));
    if !toolchain.is_file() || !installed.join(&triplet).join("lib").is_dir() {
        return Err("KiriKiri requires the prepared vcpkg installation; run cargo xtask deps".into());
    }
    let bison = PathBuf::from(env::var_os("BISON_EXECUTABLE").ok_or(
        "BISON_EXECUTABLE is missing; run cargo xtask deps for kirikiroid2-web",
    )?);
    if !bison.is_file() {
        return Err(format!("Bison executable missing: {}; run cargo xtask deps", bison.display()).into());
    }
    println!("cargo:rerun-if-changed={}", toolchain.display());
    println!("cargo:rerun-if-changed={}", bison.display());
    let mut definitions = vec![
        ("KRKR2_SHINKI_EMU".into(), "ON".into()),
        ("KRKR2_SHINKI_EMU_STATIC".into(), if cfg!(feature = "static") { "ON" } else { "OFF" }.into()),
        ("ENABLE_TESTS".into(), "OFF".into()),
        ("BUILD_TOOLS".into(), "OFF".into()),
        ("KRKR2_USE_STUB_SPDLOG".into(), "OFF".into()),
        ("CMAKE_TOOLCHAIN_FILE".into(), toolchain.to_string_lossy().replace('\\', "/")),
        ("VCPKG_INSTALLED_DIR".into(), installed.to_string_lossy().replace('\\', "/")),
        ("BISON_EXECUTABLE".into(), bison.to_string_lossy().replace('\\', "/")),
    ];
    if let Some(ffmpeg) = env::var_os("SHINKI_EMU_FFMPEG_INSTALLED") {
        let ffmpeg = PathBuf::from(ffmpeg);
        definitions.push(("CMAKE_PREFIX_PATH".into(), ffmpeg.to_string_lossy().replace('\\', "/")));
        definitions.push(("FFMPEG_DIR".into(), ffmpeg.join("lib/cmake/ffmpeg").to_string_lossy().replace('\\', "/")));
    }
    if target == "x86_64-pc-windows-msvc" { definitions.push(("WINDOWS".into(), "ON".into())); }
    if target.contains("linux") && !target.contains("android") { definitions.push(("LINUX".into(), "ON".into())); }
    if target.contains("apple") { definitions.push(("APPLE".into(), "ON".into())); }
    if target.contains("android") {
        let arch = env::var("CARGO_CFG_TARGET_ARCH")?;
        let abi = match arch.as_str() {
            "aarch64" => "arm64-v8a",
            "x86_64" => "x86_64",
            other => return Err(format!("unsupported Android architecture {other}").into()),
        };
        definitions.push(("ANDROID".into(), "ON".into()));
        definitions.push(("ANDROID_ABI".into(), abi.into()));
        definitions.push(("ANDROID_PLATFORM".into(), "android-28".into()));
    }
    if target.contains("ios") { definitions.push(("IOS".into(), "ON".into())); }
    let android_static = cfg!(feature = "static") && target.contains("android");
    if android_static {
        let query = PathBuf::from(env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?)
            .join("native/.cmake/api/v1/query");
        fs::create_dir_all(&query)?;
        fs::write(query.join("codemodel-v2"), "")?;
    }
    let native = configure_cmake(root, &definitions, &["shinki_emu_kirikiroid2_web"])?;
    let directory = native.join("shinki");
    let (library_name, static_link) = if target == "x86_64-pc-windows-msvc" {
        ("shinki_emu_kirikiroid2_web.dll", false)
    } else if target.contains("ios") || android_static {
        ("libshinki_emu_kirikiroid2_web.a", true)
    } else if target.contains("apple") { ("libshinki_emu_kirikiroid2_web.dylib", false) }
    else { ("libshinki_emu_kirikiroid2_web.so", false) };
    if !directory.join(library_name).is_file() {
        return Err(format!("CMake did not produce {}", directory.join(library_name).display()).into());
    }
    let lifecycle = if target == "x86_64-pc-windows-msvc" {
        "shinki_emu_kirikiroid2_web_lifecycle_test.lib"
    } else {
        "libshinki_emu_kirikiroid2_web_lifecycle_test.a"
    };
    if !directory.join(lifecycle).is_file() {
        return Err("CMake did not produce lifecycle test library".into());
    }
    let library = if android_static {
        android_static::link(&native, &directory.join(library_name))?
    } else {
        println!("cargo:rustc-link-search=native={}", directory.display());
        println!("cargo:rustc-link-lib={}={}", if static_link { "static" } else { "dylib" }, "shinki_emu_kirikiroid2_web");
        directory.join(library_name)
    };
    let mut manifest = CoreBuildManifest::new("kirikiroid2-web", if static_link { "native-static" } else { "native-shared" })?;
    manifest.native_library = Some(library);
    manifest.resources.push(BuildAsset {
        source: directory.join("Resources"),
        dest: "Resources".into(),
    });
    for (source, name) in [
        ("LICENSE", "LICENSE"),
        ("shinki/README.md", "adapter-README.md"),
        ("cpp/plugins/layerExImage/LICENSE.TXT", "layerExImage-LICENSE.TXT"),
        ("ui/cocos-studio/DroidSansFallback.LICENSE.txt", "DroidSansFallback.LICENSE.txt"),
        ("shinki/licenses/libbpg-MIT.txt", "libbpg-MIT.txt"),
        ("shinki/licenses/minizip-zlib.txt", "minizip-zlib.txt"),
        ("shinki/licenses/UnRAR.txt", "UnRAR.txt"),
        ("shinki/licenses/FMOD.txt", "FMOD.txt"),
    ] {
        manifest.licenses.push(BuildAsset {
            source: root.join(source),
            dest: format!("licenses/kirikiroid2-web/{name}"),
        });
    }
    let share = installed.join(&triplet).join("share");
    if share.is_dir() {
        let mut ports = fs::read_dir(&share)?.collect::<Result<Vec<_>, _>>()?;
        ports.sort_by_key(|port| port.file_name());
        for port in ports {
            let copyright = port.path().join("copyright");
            if copyright.is_file() { manifest.licenses.push(BuildAsset { source: copyright, dest: format!("licenses/kirikiroid2-web/vcpkg/{}/copyright", port.file_name().to_string_lossy()) }); }
        }
    }
    write_core_build_manifest(&manifest)?;
    Ok(())
}

fn main() {
    if let Err(error) = build() {
        panic!("KiriKiri native core build failed: {error}");
    }
}
