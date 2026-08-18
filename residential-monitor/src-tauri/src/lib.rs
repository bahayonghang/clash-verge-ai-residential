pub mod accounting;
pub mod bench;
pub mod c0_contract;
pub mod candidate_schema;
pub mod controller;
pub mod credential;
pub mod evidence;
pub mod identity;
pub mod live;
pub mod session;
pub mod sqlite_probe;
pub mod storage;
pub mod transport;
pub mod workload;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("启动家宽流量监控失败");
}
