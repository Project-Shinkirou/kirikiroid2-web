use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let output = args.next().ok_or("usage: stage <output-dir>")?;
    if args.next().is_some() {
        return Err("usage: stage <output-dir>".into());
    }
    shinki_kirikiroid2_web_android_host::stage(&PathBuf::from(output))
}
