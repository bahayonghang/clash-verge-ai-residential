//! 只采样当前基准进程；不枚举安装程序或 WebView。

use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct ProcessSample {
    pub cpu_seconds: Option<f64>,
    pub working_set_bytes: Option<u64>,
    pub private_bytes: Option<u64>,
    pub handles: Option<u32>,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
    pub io_write_operations: Option<u64>,
}

#[cfg(windows)]
pub fn sample() -> ProcessSample {
    use std::ffi::c_void;
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

    #[repr(C)]
    #[derive(Default)]
    struct Memory {
        cb: u32,
        page_fault_count: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged_pool: usize,
        quota_paged_pool: usize,
        quota_peak_non_paged_pool: usize,
        quota_non_paged_pool: usize,
        pagefile: usize,
        peak_pagefile: usize,
        private_usage: usize,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Io {
        read_operations: u64,
        write_operations: u64,
        other_operations: u64,
        read_bytes: u64,
        write_bytes: u64,
        other_bytes: u64,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn K32GetProcessMemoryInfo(process: *mut c_void, memory: *mut Memory, cb: u32) -> i32;
        fn GetProcessIoCounters(process: *mut c_void, counters: *mut Io) -> i32;
        fn GetProcessHandleCount(process: *mut c_void, count: *mut u32) -> i32;
    }
    let mut out = ProcessSample::default();
    // SAFETY: 结构遵循 Win32 ABI，缓冲区均在调用期间存活；只读取当前进程。
    unsafe {
        let process = GetCurrentProcess();
        let mut creation: FILETIME = std::mem::zeroed();
        let mut exit: FILETIME = std::mem::zeroed();
        let mut kernel: FILETIME = std::mem::zeroed();
        let mut user: FILETIME = std::mem::zeroed();
        if GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) != 0 {
            let ticks =
                |v: FILETIME| (u64::from(v.dwHighDateTime) << 32) | u64::from(v.dwLowDateTime);
            out.cpu_seconds = Some((ticks(kernel) + ticks(user)) as f64 / 10_000_000.0);
        }
        let mut memory = Memory {
            cb: std::mem::size_of::<Memory>() as u32,
            ..Memory::default()
        };
        if K32GetProcessMemoryInfo(process, &mut memory, std::mem::size_of::<Memory>() as u32) != 0
        {
            out.working_set_bytes = Some(memory.working_set as u64);
            out.private_bytes = Some(memory.private_usage as u64);
        }
        let mut io = Io::default();
        if GetProcessIoCounters(process, &mut io) != 0 {
            out.io_read_bytes = Some(io.read_bytes);
            out.io_write_bytes = Some(io.write_bytes);
            out.io_write_operations = Some(io.write_operations);
        }
        let mut handles = 0;
        if GetProcessHandleCount(process, &mut handles) != 0 {
            out.handles = Some(handles);
        }
    }
    out
}

#[cfg(not(windows))]
pub fn sample() -> ProcessSample {
    // 尚无经验证的其它平台探针；缺失数据保持 null，不能伪造为零。
    ProcessSample::default()
}
