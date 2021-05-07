use chrono::Local;
use colored::*;

fn get_time_stamp() -> String {
    Local::now().format("%y-%m-%d %H:%M:%S").to_string()
}

pub trait PlsLogger {
    fn info(&self, msg: &str);
    fn error(&self, msg: &str);
    fn warning(&self, msg: &str);
}

pub struct ColoredConsoleLogger;

impl PlsLogger for ColoredConsoleLogger {
    fn info(&self, msg: &str) {
        println!(
            "{} {}",
            get_time_stamp().white(),
            format!("info: {}", msg).green(),
        );
    }

    fn error(&self, msg: &str) {
        println!(
            "{} {}",
            get_time_stamp().white(),
            format!("error: {}", msg).red(),
        );
    }

    fn warning(&self, msg: &str) {
        println!(
            "{} {}",
            get_time_stamp().white(),
            format!("warning: {}", msg).yellow(),
        );
    }
}
