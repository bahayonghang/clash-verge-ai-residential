//! C2 桌面外壳：只消费 C1 冻结接口，不打开 rusqlite。

pub mod close;
pub mod collector;
pub mod contract;
pub mod desktop;
pub mod dialog;
pub mod facade;
pub mod hub;
pub mod query;
pub mod settings;
pub mod shell;
pub mod subscriptions;
