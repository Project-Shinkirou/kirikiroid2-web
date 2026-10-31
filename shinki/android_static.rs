//! Resolve CMake's real link plan, then namespace the entire native closure.
//! Undefined Android/C++ runtime imports stay untouched; every defined private
//! symbol (and its internal references) receives the same core-specific name.
use serde_json::Value;
use std::{collections::{BTreeMap, BTreeSet}, env, error::Error, fs, path::{Path, PathBuf}, process::Command};

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;
const EXPORTS: [&str; 3] = ["shinki_emu_kirikiroid2_web_query", "shinki_emu_kirikiroid2_web_set_resource_root", "shinki_emu_kirikiroid2_web_android_initialize"];

fn capture(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!("{command:?}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
fn json(path: &Path) -> Result<Value> { Ok(serde_json::from_slice(&fs::read(path)?)?) }

#[path = "android_link_plan.rs"]
mod link_plan;
enum LinkInput { Archive(usize), Whole(bool) }
pub fn link(native: &Path, adapter: &Path) -> Result<PathBuf> {
    let reply = native.join(".cmake/api/v1/reply");
    let mut indices = fs::read_dir(&reply)?.collect::<std::io::Result<Vec<_>>>()?;
    indices.retain(|entry| entry.file_name().to_string_lossy().starts_with("index-"));
    indices.sort_by_key(|entry| entry.file_name());
    let index = json(&indices.last().ok_or("CMake File API index missing")?.path())?;
    let model = json(&reply.join(index["reply"]["codemodel-v2"]["jsonFile"].as_str().ok_or("CMake codemodel missing")?))?;
    let targets = model["configurations"][0]["targets"].as_array().ok_or("CMake targets missing")?;
    let plan = targets.iter().find(|target| target["name"] == "shinki_krkr_link_plan").ok_or("Krkr link plan missing")?;
    let plan = json(&reply.join(plan["jsonFile"].as_str().ok_or("link plan JSON missing")?))?;
    let generator = index["cmake"]["generator"]["name"].as_str().ok_or("CMake generator missing")?;
    let directory = link_plan::command_directory(native, plan["paths"]["build"].as_str().ok_or("link plan build directory missing")?, generator)?;
    let mut project_artifacts = BTreeSet::new();
    for target in targets {
        let target = json(&reply.join(target["jsonFile"].as_str().ok_or("CMake target file missing")?))?;
        if target["type"] != "STATIC_LIBRARY" { continue; }
        for artifact in target["artifacts"].as_array().ok_or("CMake archive artifacts missing")? {
            let path = native.join(artifact["path"].as_str().ok_or("CMake artifact path missing")?);
            if path.is_file() { project_artifacts.insert(fs::canonicalize(path)?); }
        }
    }
    let resolve = |token: &str| link_plan::resolve_archive(token, &directory, &project_artifacts);
    let mut tokens = Vec::new();
    for fragment in plan["link"]["commandFragments"].as_array().ok_or("CMake link fragments missing")? {
        let text = fragment["fragment"].as_str().ok_or("link fragment missing")?;
        #[cfg(windows)]
        let arguments = link_plan::windows_arguments(text)?;
        #[cfg(not(windows))]
        let arguments = shlex::split(text).ok_or("invalid POSIX CMake link fragment")?;
        tokens.extend(arguments);
    }
    let mut search = Vec::new();
    for token in &tokens {
        if let Some(path) = token.strip_prefix("-L") {
            let path = PathBuf::from(path);
            if !path.is_absolute() || !path.is_dir() { return Err(format!("invalid CMake library directory: {}", path.display()).into()); }
            if !search.contains(&path) { search.push(path); }
        }
    }
    for token in tokens.iter().filter(|token| token.ends_with(".a")) {
        let path = resolve(token)?;
        let parent = path.parent().ok_or("archive parent missing")?.to_owned();
        if !search.contains(&parent) { search.push(parent); }
    }
    let mut archives = vec![adapter.to_owned()];
    let mut archive_ids = BTreeMap::from([(fs::canonicalize(adapter)?, 0)]);
    let mut inputs = vec![LinkInput::Archive(0)];
    let mut system = BTreeSet::new();
    for token in tokens {
        if matches!(token.as_str(), "-Wl,--whole-archive" | "--whole-archive") {
            inputs.push(LinkInput::Whole(true));
            continue;
        }
        if matches!(token.as_str(), "-Wl,--no-whole-archive" | "--no-whole-archive") {
            inputs.push(LinkInput::Whole(false));
            continue;
        }
        let path = if token.ends_with(".a") {
            resolve(&token)?
        } else if let Some(name) = token.strip_prefix("-l") {
            if matches!(name, "android" | "log" | "dl" | "m" | "c" | "atomic" | "EGL" | "GLESv1_CM" | "GLESv2" | "OpenSLES" | "c++_shared" | "omp") {
                system.insert(name.to_owned());
                continue;
            }
            // pkg-config emits some transitive native libraries as -l names.
            // They belong inside the same private static closure, not in Rust's
            // process-global/system link set.
            search.iter().map(|dir| dir.join(format!("lib{name}.a"))).find(|path| path.is_file())
                .ok_or_else(|| format!("CMake native library has no static archive: {name}"))?
        } else if token.ends_with(".so") {
            let name = Path::new(&token).file_name().and_then(|s| s.to_str()).ok_or("runtime filename")?;
            if !matches!(name, "libc++_shared.so" | "libomp.so") { return Err(format!("non-static Krkr dependency: {token}").into()); }
            println!("cargo:rustc-link-search=native={}", Path::new(&token).parent().unwrap().display());
            system.insert(name.trim_start_matches("lib").trim_end_matches(".so").to_owned());
            continue;
        } else { continue; };
        let identity = fs::canonicalize(&path)?;
        let index = if let Some(&index) = archive_ids.get(&identity) {
            index
        } else {
            let index = archives.len();
            archives.push(path);
            archive_ids.insert(identity, index);
            index
        };
        inputs.push(LinkInput::Archive(index));
    }
    let ndk = PathBuf::from(env::var_os("ANDROID_NDK_HOME").ok_or("ANDROID_NDK_HOME missing")?);
    let host = if cfg!(windows) { "windows-x86_64" } else if cfg!(target_os = "macos") { "darwin-x86_64" } else { "linux-x86_64" };
    let tools = ndk.join("toolchains/llvm/prebuilt").join(host).join("bin");
    let executable = |name: &str| tools.join(format!("{name}{}", if cfg!(windows) { ".exe" } else { "" }));
    for input in &mut archives {
        if input.file_name().is_some_and(|name| name == "libSDL2.a") {
            let private = native.join("shinki/libSDL2-private.a");
            capture(Command::new(executable("llvm-objcopy"))
                .arg("--redefine-sym=JNI_OnLoad=shinki_krkr_sdl_on_load")
                .arg(&*input).arg(&private))?;
            *input = private;
        }
    }
    let object = native.join("shinki/krkr-closure.o");
    let response = native.join("shinki/krkr-closure.rsp");
    let mut args = vec!["-r".to_owned(), "--start-group".to_owned()];
    for input in inputs {
        args.push(match input {
            LinkInput::Archive(index) => format!("\"{}\"", archives[index].to_string_lossy().replace('\\', "/")),
            LinkInput::Whole(true) => "--whole-archive".into(),
            LinkInput::Whole(false) => "--no-whole-archive".into(),
        });
    }
    args.push("--end-group".into());
    for export in EXPORTS { args.push(format!("--undefined={export}")); }
    args.extend(["-o".into(), format!("\"{}\"", object.to_string_lossy().replace('\\', "/"))]);
    fs::write(&response, args.join("\n"))?;
    capture(Command::new(executable("ld.lld")).arg(format!("@{}", response.display())))?;
    // Keep private Java class names and native RegisterNatives tables aligned.
    // Equal-width replacements preserve ELF section offsets and relocations.
    let mut bytes = fs::read(&object)?;
    for (before, after) in [
        (b"org/libsdl/app".as_slice(), b"org/krkrsd/app".as_slice()),
        (b"org.libsdl.app".as_slice(), b"org.krkrsd.app".as_slice()),
        (b"org/cocos2dx".as_slice(), b"org/krkrc2dx".as_slice()),
        (b"org.cocos2dx".as_slice(), b"org.krkrc2dx".as_slice()),
    ] {
        assert_eq!(before.len(), after.len());
        let mut offset = 0;
        while let Some(index) = bytes[offset..].windows(before.len()).position(|window| window == before) {
            let start = offset + index;
            bytes[start..start + before.len()].copy_from_slice(after);
            offset = start + before.len();
        }
    }
    fs::write(&object, bytes)?;
    let symbols = capture(Command::new(executable("llvm-nm")).args(["--defined-only", "--extern-only", "--format=posix"]).arg(&object))?;
    let mut renames = BTreeSet::new();
    for line in symbols.lines() {
        let symbol = line.split_whitespace().next().ok_or("invalid nm symbol")?;
        if !EXPORTS.contains(&symbol) { renames.insert(format!("{symbol} shinki_krkr_{symbol}")); }
    }
    let map = native.join("shinki/krkr-symbols.map");
    fs::write(&map, renames.into_iter().collect::<Vec<_>>().join("\n") + "\n")?;
    let isolated = native.join("shinki/krkr-isolated.o");
    capture(Command::new(executable("llvm-objcopy")).arg(format!("--redefine-syms={}", map.display())).arg(&object).arg(&isolated))?;
    let isolated_symbols = capture(Command::new(executable("llvm-nm")).args(["--defined-only", "--extern-only", "--format=posix"]).arg(&isolated))?;
    let names: BTreeSet<_> = isolated_symbols.lines().filter_map(|line| line.split_whitespace().next()).collect();
    if EXPORTS.iter().any(|export| !names.contains(export))
        || names.iter().any(|name| !EXPORTS.contains(name) && !name.starts_with("shinki_krkr_")) {
        return Err("Krkr static closure leaked an unisolated symbol or lost its query/resource entry".into());
    }
    let archive = native.join("shinki/libshinki_krkr_closure.a");
    if archive.exists() { fs::remove_file(&archive)?; }
    capture(Command::new(executable("llvm-ar")).arg("crs").arg(&archive).arg(&isolated))?;
    fs::write(native.join("shinki/krkr-static-inputs.json"), serde_json::to_vec_pretty(&archives)?)?;
    println!("cargo:rustc-link-search=native={}", archive.parent().unwrap().display());
    println!("cargo:rustc-link-lib=static=shinki_krkr_closure");
    system.extend(["c++_shared", "android", "log", "dl", "m", "EGL", "GLESv2", "GLESv1_CM", "OpenSLES"].map(str::to_owned));
    for name in system { println!("cargo:rustc-link-lib={name}"); }
    Ok(archive)
}
