pub mod napcat;

#[cfg(feature = "napcat")]
pub use napcat::Message;

#[cfg(feature = "napcat")]
pub use napcat::MsgType;


