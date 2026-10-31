//! Resolve CMake's real link plan, then namespace the entire native closure.
//! Undefined Android/C++ runtime imports stay untouched; every defined private
//! symbol (and its internal references) receives the same core-specific name.
use serde_json::Value;
use std::{collections::BTreeSet, env, error::Error, fs, path::{Path, PathBuf}, process::Command};

type Result<T, E = Box<dyn Error>> = std::result::Result<T, E>;
const EXPORTS: [&str; 2] = ["shinki_emu_kirikiroid2_web_query", "shinki_emu_kirikiroid2_web_set_resource_root"];

fn capture(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(format!("{command:?}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
fn json(path: &Path) -> Result<Value> { Ok(serde_json::from_slice(&fs::read(path)?)?) }

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
    let mut archives = vec![adapter.to_owned()];
    let mut system = BTreeSet::new();
    for fragment in plan["link"]["commandFragments"].as_array().ok_or("CMake link fragments missing")? {
        if fragment["role"] != "libraries" { continue; }
        for token in shlex::split(fragment["fragment"].as_str().ok_or("link fragment missing")?).ok_or("invalid CMake link fragment")? {
            if token.ends_with(".a") {
                let path = PathBuf::from(&token);
                let path = if path.is_absolute() { path } else { native.join(path) };
                if !path.is_file() { return Err(format!("static dependency missing: {}", path.display()).into()); }
                if !archives.contains(&path) { archives.push(path); }
            } else if let Some(name) = token.strip_prefix("-l") {
                system.insert(name.to_owned());
            } else if token.ends_with(".so") {
                // Only toolchain runtimes may remain shared. A core dependency
                // must be built statically, not silently shipped as a plugin.
                let name = Path::new(&token).file_name().and_then(|s| s.to_str()).ok_or("runtime filename")?;
                if !matches!(name, "libc++_shared.so" | "libomp.so") {
                    return Err(format!("non-static Krkr dependency: {token}").into());
                }
                println!("cargo:rustc-link-search=native={}", Path::new(&token).parent().unwrap().display());
                system.insert(name.trim_start_matches("lib").trim_end_matches(".so").to_owned());
            }
        }
    }
    let ndk = PathBuf::from(env::var_os("ANDROID_NDK_HOME").ok_or("ANDROID_NDK_HOME missing")?);
    let host = if cfg!(windows) { "windows-x86_64" } else if cfg!(target_os = "macos") { "darwin-x86_64" } else { "linux-x86_64" };
    let tools = ndk.join("toolchains/llvm/prebuilt").join(host).join("bin");
    let executable = |name: &str| tools.join(format!("{name}{}", if cfg!(windows) { ".exe" } else { "" }));
    let object = native.join("shinki/krkr-closure.o");
    let response = native.join("shinki/krkr-closure.rsp");
    let mut args = vec!["-r".to_owned(), "--start-group".to_owned()];
    args.extend(archives.iter().map(|path| format!("\"{}\"", path.to_string_lossy().replace('\\', "/"))));
    args.push("--end-group".into());
    for export in EXPORTS { args.push(format!("--undefined={export}")); }
    args.extend(["-o".into(), format!("\"{}\"", object.to_string_lossy().replace('\\', "/"))]);
    fs::write(&response, args.join("\n"))?;
    capture(Command::new(executable("ld.lld")).arg(format!("@{}", response.display())))?;
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
