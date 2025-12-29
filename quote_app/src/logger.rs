use std::fs::File;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

pub(crate) fn setup_logger(level: LevelFilter, file: &str) {
    let log_file = File::create(file).expect("Error create log file");
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}:{} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .target(Target::Pipe(Box::new(log_file)))
        .filter(None, level) // Уровень по умолчанию
        .write_style(env_logger::WriteStyle::Always) // Всегда использовать цвета
        .init();
}