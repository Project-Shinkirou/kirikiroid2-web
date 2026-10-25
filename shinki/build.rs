use shinki_emu_build::{
    configure_cmake, write_core_build_manifest, BuildAsset, CoreBuildManifest,
};
use std::{env, fs, path::PathBuf};

fn build() -> Result<(), Box<dyn std::error::Error>> {
    let target = env::var("TARGET")?;
    let host = env::var("HOST")?;
    if !matches!(target.as_str(), "x86_64-pc-windows-msvc" | "x86_64-unknown-linux-gnu") || host != target {
        return Err(format!("KiriKiri requires a native Windows x64/MSVC or Linux x64/GNU build; HOST={host}, TARGET={target}").into());
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
    let installed = PathBuf::from(env::var_os("SHINKI_EMU_VCPKG_INSTALLED").ok_or(
        "SHINKI_EMU_VCPKG_INSTALLED is missing; run cargo xtask deps for kirikiroid2-web",
    )?);
    let triplet = env::var("SHINKI_EMU_VCPKG_TRIPLET")?;
    let windows = target.ends_with("windows-msvc");
    let expected_triplet = if windows { "x64-windows" } else { "x64-linux-dynamic" };
    if triplet != expected_triplet || !toolchain.is_file() || !installed.join(&triplet).join("lib").is_dir() {
        return Err(format!("KiriKiri requires the prepared {expected_triplet} vcpkg installation; run cargo xtask deps").into());
    }
    let bison = PathBuf::from(env::var_os("BISON_EXECUTABLE").ok_or(
        "BISON_EXECUTABLE is missing; run cargo xtask deps for kirikiroid2-web",
    )?);
    if !bison.is_file() {
        return Err(format!("Bison executable missing: {}; run cargo xtask deps", bison.display()).into());
    }
    println!("cargo:rerun-if-changed={}", toolchain.display());
    println!("cargo:rerun-if-changed={}", bison.display());
    let definitions = vec![
        (if windows { "WINDOWS" } else { "LINUX" }.into(), "ON".into()),
        ("KRKR2_SHINKI_EMU".into(), "ON".into()),
        ("ENABLE_TESTS".into(), "OFF".into()),
        ("BUILD_TOOLS".into(), "OFF".into()),
        ("KRKR2_USE_STUB_SPDLOG".into(), "OFF".into()),
        ("CMAKE_TOOLCHAIN_FILE".into(), toolchain.to_string_lossy().replace('\\', "/")),
        ("BISON_EXECUTABLE".into(), bison.to_string_lossy().replace('\\', "/")),
    ];
    let native = configure_cmake(root, &definitions, &["shinki_emu_kirikiroid2_web"])?;
    let directory = native.join("shinki");
    let library = if windows { "shinki_emu_kirikiroid2_web.dll" } else { "libshinki_emu_kirikiroid2_web.so" };
    let required = if windows {
        vec![library, "shinki_emu_kirikiroid2_web.lib", "shinki_emu_kirikiroid2_web_lifecycle_test.lib"]
    } else {
        vec![library, "libshinki_emu_kirikiroid2_web_lifecycle_test.a"]
    };
    for name in required {
        if !directory.join(name).is_file() {
            return Err(format!("CMake did not produce {}", directory.join(name).display()).into());
        }
    }
    println!("cargo:rustc-link-search=native={}", directory.display());
    println!("cargo:rustc-link-lib=dylib=shinki_emu_kirikiroid2_web");
    let mut manifest = CoreBuildManifest::new("kirikiroid2-web", "native-shared")?;
    manifest.native_library = Some(directory.join(library));
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
    ] {
        manifest.licenses.push(BuildAsset {
            source: root.join(source),
            dest: format!("licenses/kirikiroid2-web/{name}"),
        });
    }
    let share = installed.join(triplet).join("share");
    let mut ports = fs::read_dir(&share)?.collect::<Result<Vec<_>, _>>()?;
    ports.sort_by_key(|port| port.file_name());
    for port in ports {
        let copyright = port.path().join("copyright");
        if copyright.is_file() {
            manifest.licenses.push(BuildAsset {
                source: copyright,
                dest: format!("licenses/kirikiroid2-web/vcpkg/{}/copyright", port.file_name().to_string_lossy()),
            });
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
