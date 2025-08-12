use std::{
    env::current_exe, fmt::Display, os::windows::process::CommandExt, path::PathBuf, process::Command
};

use bitflags::bitflags;
use clap::{Parser, ValueEnum};
use uuid::Uuid;

bitflags! {
    pub struct CreationFlags: u32 {
        const DEBUG_PROCESS             = 0x00000001;
        const DEBUG_ONLY_THIS_PROCESS   = 0x00000002;
        const SUSPENDED                 = 0x00000004;
        const DETACHED_PROCESS          = 0x00000008;
        const NEW_CONSOLE               = 0x00000010;
        const NEW_PROCESS_GROUP         = 0x00000200;
        const SEPARATE_WOW_VDM          = 0x00000800;
        const SHARED_WOW_VDM            = 0x00001000;
        const INHERIT_PARENT_AFFINITY   = 0x00010000;
        const PROTECTED_PROCESS         = 0x00040000;
        const BREAKAWAY_FROM_JOB        = 0x01000000;
        const DEFAULT_ERROR_MODE        = 0x04000000;
        const NO_WINDOW                 = 0x08000000;
        const SECURE_PROCESS            = 0x00400000;
        const UNICODE_ENVIRONMENT       = 0x00000400;
        const PRESERVE_CODE_AUTHZ_LEVEL = 0x02000000;
    }
}

impl Into<u32> for CreationFlags {
    fn into(self) -> u32 {
        self.bits()
    }
}

const PROFILER_CLSID: &str = "DF63A541-5A33-4611-8829-F4E495985EE3";
const PROFILER_DLL: &str = "test_profilers.dll";

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Create process in suspended mode
    /// 
    /// Useful for debugging. Only works with EXEs.
    #[arg(short, long)]
    suspended: bool,

    /// Profiler (DLL) to use
    #[arg(short, long, default_value = PROFILER_DLL )]
    profiler: PathBuf,
    
    /// GUID of profiler interface
    /// 
    /// Ideally a CLSID is required for profiler interface, 
    /// but implementation may choose to not check.
    #[arg(short, long, 
        default_value = PROFILER_CLSID, 
        value_parser = Uuid::parse_str
    )]
    clsid: Uuid,

    /// Log level to use
    /// 
    /// Sets RUST_LOG environment variable.
    #[arg(short, long, ignore_case=true, default_value_t=Default::default())]
    log_level: LogLevel,

    /// .NET program to profile
    target: PathBuf,

    /// Arguments to the target
    #[arg(last = true)]
    params: Vec<String>
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy)]
#[derive(ValueEnum)]
enum LogLevel {
    Off   = 0,
    Error = 1,
    Warn  = 2,
    #[default]
    Info  = 3,
    Debug = 4,
    Trace = 5,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn get_profiler_path(profiler_path: &PathBuf) -> PathBuf {
    let exe_path = current_exe().expect("failed to get current exe");
    let exe_dir = exe_path.parent().expect(&format!("failed to get parent of {}", exe_path.display()));
    let profiler_path = exe_dir.join(profiler_path);
    profiler_path.canonicalize().expect(&format!("failed to make absolute path from {}", profiler_path.display()))
}

fn launch_with_profiler(profiler_path: &PathBuf, command_line: &str, opts: &Cli) {
    let mut command = Command::new(command_line);

    if opts.suspended {
        println!("Luanching in suspended mode. Attach debugger to target and continue.");
        command.creation_flags(CreationFlags::SUSPENDED.into())
    } else { 
        &mut command
    }
        .env("RUST_LOG", opts.log_level.to_string())
        .env("COR_ENABLE_PROFILING", "1")
        .env("COR_PROFILER", format!("{{{}}}", opts.clsid.to_hyphenated().to_string()))
        .env("COR_PROFILER_PATH", 
            profiler_path.to_str()
                .unwrap_or_else(|| panic!("can't convert profiler path to &str")
            )
        );

    match command.status() {
        Ok(status) => {
            println!("process existed with status {status}");
        },
        Err(err) => {
            println!("failed to launch '{command_line}' with profiler; err: {err}");
        }
    }
}


fn get_command_line(target: &PathBuf, args: &[String]) -> String {
    let target_path = target.canonicalize().unwrap();

    if !target_path.is_file() {
        panic!("{} is not a file", target_path.display());
    }

    let target_path = target_path.to_str().unwrap();
    if target.ends_with(".dll") {
        format!("dotnet {}", target_path).to_string()
    } else {
        let mut cmdl = vec![target_path.to_string()];
        cmdl.extend(args.iter().cloned()); // append arguments
        cmdl.join(" ")
    }
}

fn main() {
    let args = Cli::parse();
    eprintln!("{args:#?}");
    eprintln!("RUST_LOG={:?}", args.log_level as u8);

    let profiler_path = get_profiler_path(&args.profiler);
    let cmdline = get_command_line(&args.target, &args.params);

    println!("[*] launching '{}' with profiler {}", cmdline, profiler_path.display());
    launch_with_profiler(&profiler_path, &cmdline, &args);
}
