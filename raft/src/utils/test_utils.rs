use std::sync::Once;

static INIT: Once = Once::new();

/// Set up the SimpleLogger.
/// No matter how many times this function is invoked,
/// the actual logger is only set up once.
pub fn set_up_logging() {
    INIT.call_once(|| {
        simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Trace).init().unwrap();
    });
}