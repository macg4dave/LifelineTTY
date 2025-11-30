pub mod sync;
pub mod backoff;
#[cfg(feature = "async-serial")]
pub mod r#async;

pub use sync::SerialPort;
