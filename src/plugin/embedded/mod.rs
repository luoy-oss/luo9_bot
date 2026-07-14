// src/plugin/embedded/mod.rs
//! 嵌入式运行时模块
//!
//! 各语言运行时通过 feature flag 条件编译，未启用的语言不增加依赖。

#[cfg(feature = "python-plugin")]
pub mod python;

#[cfg(feature = "java-plugin")]
pub mod jvm;

#[cfg(feature = "quickjs-plugin")]
pub mod quickjs;
