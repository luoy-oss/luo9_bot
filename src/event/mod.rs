
pub mod napcat;

#[cfg(feature = "napcat")]
pub use napcat::{MetaEvent, MetaEventType};

#[cfg(feature = "napcat")]
pub use napcat::PostType; 
