use std::io::stdout;

use fern::log_file;
use log::debug;

pub fn init_logging(){
    let file_name = "prof_info.log";
    match log_file(file_name) {
        Ok(file_dispatch) =>
            match fern::Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!("{} [{}]-[{}#{}] {}",
                    record.level(),
                    record.target(),
                    record.file().unwrap_or("-"),
                    record.line().unwrap_or(0),
                    message
                ));
            })
            .level(log::LevelFilter::Debug)
            .chain(stdout())
            .chain(file_dispatch)
            .apply() 
            {
                Ok(_) => debug!("logging initialized"),
                Err(err) => eprintln!("init_logging failed with {err}"),
            },
        Err(err) => eprintln!("failed to create log file '{file_name}'; err {err}")
    }
}
