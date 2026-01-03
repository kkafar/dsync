use log4rs::{
    Config,
    append::console::ConsoleAppender,
    config::{Appender, Logger, Root},
    encode::pattern::PatternEncoder,
};

pub(crate) fn configure_logging(log_level: log::LevelFilter) -> log4rs::Handle {
    // https://docs.rs/log4rs/latest/log4rs/encode/pattern/index.html
    let base_encoder = Box::new(PatternEncoder::new(
        "{date} {highlight({level})} {target} - {message}{n}",
    ));

    let console_appender_name = "stdout";
    let console_appender = ConsoleAppender::builder().encoder(base_encoder).build();

    let peer_service_logger = Logger::builder()
        .appender(console_appender_name)
        .build("pslog", log_level);

    let logging_config = Config::builder()
        .appender(Appender::builder().build(console_appender_name, Box::new(console_appender)))
        .logger(peer_service_logger)
        .build(
            Root::builder()
                .appender(console_appender_name)
                .build(log_level),
        )
        .unwrap();

    let handle = log4rs::init_config(logging_config).unwrap();

    log::info!("Logging configured with level: {log_level}");
    handle
}
