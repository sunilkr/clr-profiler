use std::{
    env::{args, current_exe}, 
    ffi::OsStr, 
    path::PathBuf, 
    process::Command, str::FromStr
};

const PROFILER_CLSID: &str = "DF63A541-5A33-4611-8829-F4E495985EE3";
const PROFILER_DLL: &str = "test_profilers.dll";

fn get_profiler_path() -> PathBuf {
    let exe_path = current_exe().expect("failed to get current exe");
    let exe_dir = exe_path.parent().expect(&format!("failed to get parent of {}", exe_path.display()));
    let profiler_path = exe_dir.join(PROFILER_DLL);
    profiler_path.canonicalize().expect(&format!("failed to make absolute path from {}", profiler_path.display()))
}

fn launch_with_profiler(target: &str, profiler_path: &PathBuf) {
    let target_path = PathBuf::from_str(target)
        .unwrap()
        .canonicalize()
        .unwrap();
    let target_path = target_path.to_str().unwrap();
    let cmdline = if target.ends_with(".dll") {
        &format!("dotnet {}", target_path)
    } else {
        target_path
    };

    let mut command = Command::new(cmdline);
    command
        .env("COR_ENABLE_PROFILING", "1")
        .env("COR_PROFILER", format!("{{{}}}", PROFILER_CLSID))
        .env("COR_PROFILER_PATH", profiler_path.to_str().unwrap_or_else(|| panic!("can't convert profiler path to &str")));

    match command.status() {
        Ok(status) => {
            println!("process existed with status {status}");
        },
        Err(err) => {
            println!("failed to launch '{cmdline}' with profiler; err: {err}");
        }
    }
}

fn main() {
    let mut args = args();
    if let Some(target) = args.nth(1) {
        let profiler_path = get_profiler_path();
        println!("launching {target} with profiler {}", profiler_path.display());
        launch_with_profiler(&target, &profiler_path);
    } else {
        const DEFAULT_EXE:&str = "profile";
        let exe = current_exe().unwrap_or(DEFAULT_EXE.into());
        let exe = exe.file_name().unwrap_or(OsStr::new(DEFAULT_EXE));
        println!("Usage: {} <target>", exe.to_str().unwrap_or(DEFAULT_EXE));
    }
}
