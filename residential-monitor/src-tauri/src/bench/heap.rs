//! bench 专用堆计数。只由 monitor-bench 注册为全局分配器；产品与 monitor-db 不注册。
//! 仅 `RESIWATCH_BENCH_HEAP=1` 时计数；关闭时每次分配只多一次原子读。
//! SQLite 自身分配走 C malloc，不经过本分配器，另用 `sqlite3_status64(SQLITE_STATUS_MEMORY_USED)` 统计。

use rusqlite::ffi;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(false);
// 启用前的分配在启用后释放会让 LIVE 为负；只使用差值，不使用绝对值。
static LIVE: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicIsize = AtomicIsize::new(0);

pub struct CountingAlloc;

fn grow(bytes: usize) {
    let live = LIVE.fetch_add(bytes as isize, Ordering::Relaxed) + bytes as isize;
    PEAK.fetch_max(live, Ordering::Relaxed);
}

fn shrink(bytes: usize) {
    LIVE.fetch_sub(bytes as isize, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() && ENABLED.load(Ordering::Relaxed) {
            grow(layout.size());
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() && ENABLED.load(Ordering::Relaxed) {
            grow(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        if ENABLED.load(Ordering::Relaxed) {
            shrink(layout.size());
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new = System.realloc(ptr, layout, new_size);
        if !new.is_null() && ENABLED.load(Ordering::Relaxed) {
            if new_size >= layout.size() {
                grow(new_size - layout.size());
            } else {
                shrink(layout.size() - new_size);
            }
        }
        new
    }
}

/// 只应在已注册 `CountingAlloc` 的进程中调用。
pub fn enable_from_env() {
    if std::env::var("RESIWATCH_BENCH_HEAP").as_deref() == Ok("1") {
        ENABLED.store(true, Ordering::SeqCst);
    }
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

/// 阶段起点：把 Rust 峰值与 SQLite 高水位重置为当前值。
#[derive(Clone, Copy)]
pub struct Mark {
    pub rust: isize,
    pub sqlite: i64,
}

pub fn mark() -> Mark {
    let rust = LIVE.load(Ordering::Relaxed);
    PEAK.store(rust, Ordering::Relaxed);
    Mark {
        rust,
        sqlite: sqlite_memory(true).0,
    }
}

/// 一次读取 SQLite 当前值与高水位，`reset` 时同时把高水位重置为当前值。
fn sqlite_memory(reset: bool) -> (i64, i64) {
    let (mut current, mut highwater) = (0_i64, 0_i64);
    // SAFETY: 只读写 SQLite 全局内存统计，不需要连接；两个输出指针在调用期间有效。
    unsafe {
        ffi::sqlite3_status64(
            ffi::SQLITE_STATUS_MEMORY_USED,
            &mut current,
            &mut highwater,
            i32::from(reset),
        );
    }
    (current, highwater)
}

/// 自 `mark` 起的峰值增量（字节），返回 (Rust, SQLite)。
pub fn peak_since(mark: Mark) -> (i64, i64) {
    let rust = PEAK.load(Ordering::Relaxed) - mark.rust;
    let sqlite = sqlite_memory(false).1 - mark.sqlite;
    (rust as i64, sqlite)
}
