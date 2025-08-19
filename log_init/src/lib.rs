use std::{env, io::stdout, str::FromStr};

use chrono::Local;
use fern::log_file;
use log::debug;

pub fn init_logging(){

    let log_level = match env::var_os("RUST_LOG"){
        Some(val) => log::LevelFilter::from_str(
            val.into_string()
                .unwrap_or(String::from("info"))
                .as_str()
            )
            .unwrap_or(log::LevelFilter::Info),
        None => log::LevelFilter::Info,
    };

    let log_file_path = env::var_os("LOG_FILE")
        .unwrap_or("prof_info.log".into())
        .to_string_lossy()
        .to_string();

    match log_file(log_file_path.as_str()) {
        Ok(file_dispatch) =>
            match fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!("{} [{}] {}",
                    record.level(),
                    record.target(),
                    //record.file().unwrap_or("-"),
                    //record.line().unwrap_or(0),
                    message
                ));
            })
            .level(log_level)
            .chain(stdout())
            .chain(file_dispatch)
            .apply() 
            {
                Ok(_) => {
                    debug!("---------- starting new run at {:?} -----------", Local::now());
                    debug!("logging initialized with level {log_level}")
                },
                Err(err) => eprintln!("init_logging failed with {err}"),
            },
        Err(err) => eprintln!("failed to create log file '{log_file_path}'; err {err}")
    }
}
