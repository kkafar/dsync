use log::LevelFilter;
use log4rs::{
    Config,
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

pub(crate) fn configure_logging() -> log4rs::Handle {
    // https://docs.rs/log4rs/latest/log4rs/encode/pattern/index.html
    let base_encoder = Box::new(PatternEncoder::new(
        "{date} {highlight({level})} {target} - {message}{n}",
    ));

    let console_appender_name = "stdout";
    let console_appender = ConsoleAppender::builder().encoder(base_encoder).build();

    // let logger = Logger::builder()
    //     .appender(console_appender_name)
    //     .build("main", LevelFilter::Trace);

    let logging_config = Config::builder()
        .appender(Appender::builder().build(console_appender_name, Box::new(console_appender)))
        .build(
            Root::builder()
                .appender(console_appender_name)
                .build(LevelFilter::Trace),
        )
        .unwrap();

    let handle = log4rs::init_config(logging_config).unwrap();
    return handle;
}
