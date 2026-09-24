//! 隔离基准专用 SQLite VFS 代理，计数成功 xWrite 的请求字节。
//! 不改变 FULL、锁、同步、checkpoint 或错误返回；不代表物理磁盘写入。

use rusqlite::ffi;
use serde::Serialize;
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

static BENCH_LOCK: Mutex<()> = Mutex::new(());

#[derive(Default)]
struct Counters {
    db: AtomicU64,
    wal: AtomicU64,
    other: AtomicU64,
    writes: AtomicU64,
    failed_writes: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct WriteSample {
    pub db_bytes: u64,
    pub wal_bytes: u64,
    pub other_sqlite_file_bytes: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
}

impl WriteSample {
    pub fn since(self, before: Self) -> Self {
        Self {
            db_bytes: self.db_bytes.saturating_sub(before.db_bytes),
            wal_bytes: self.wal_bytes.saturating_sub(before.wal_bytes),
            other_sqlite_file_bytes: self
                .other_sqlite_file_bytes
                .saturating_sub(before.other_sqlite_file_bytes),
            successful_calls: self
                .successful_calls
                .saturating_sub(before.successful_calls),
            failed_calls: self.failed_calls.saturating_sub(before.failed_calls),
        }
    }
}

#[repr(C)]
struct ProxyVfs {
    base: ffi::sqlite3_vfs,
    original: *mut ffi::sqlite3_vfs,
    root: String,
    counters: Arc<Counters>,
}

#[repr(C)]
struct ProxyFile {
    base: ffi::sqlite3_file,
    inner: *mut ffi::sqlite3_file,
    counters: *const Counters,
    kind: c_int,
}

pub struct WriteVfs {
    vfs: Box<ProxyVfs>,
    _name: CString,
    _lock: MutexGuard<'static, ()>,
}

impl WriteVfs {
    pub fn install(root: &Path) -> Result<Self, String> {
        let lock = BENCH_LOCK.lock().map_err(|_| "基准 VFS 锁中毒")?;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let root = normalize_path(&root.to_string_lossy());
        let name = CString::new("resiwatch-isolated-bench").map_err(|e| e.to_string())?;
        // SAFETY: SQLite 已知默认 VFS；复制其回调，仅替换 xOpen，注册期保持 Box/CString 地址稳定。
        unsafe {
            let original = ffi::sqlite3_vfs_find(std::ptr::null());
            if original.is_null() {
                return Err("SQLite 默认 VFS 不存在".into());
            }
            let mut vfs = Box::new(ProxyVfs {
                base: std::ptr::read(original),
                original,
                root,
                counters: Arc::new(Counters::default()),
            });
            vfs.base.zName = name.as_ptr();
            vfs.base.pNext = std::ptr::null_mut();
            vfs.base.szOsFile += std::mem::size_of::<ProxyFile>() as c_int;
            vfs.base.xOpen = Some(open);
            macro_rules! wrap {
                ($field:ident, $callback:ident) => {
                    if vfs.base.$field.is_some() {
                        vfs.base.$field = Some($callback);
                    }
                };
            }
            wrap!(xDelete, vfs_delete);
            wrap!(xAccess, vfs_access);
            wrap!(xFullPathname, vfs_fullpath);
            wrap!(xDlOpen, vfs_dl_open);
            wrap!(xDlError, vfs_dl_error);
            wrap!(xDlSym, vfs_dl_sym);
            wrap!(xDlClose, vfs_dl_close);
            wrap!(xRandomness, vfs_random);
            wrap!(xSleep, vfs_sleep);
            wrap!(xCurrentTime, vfs_time);
            wrap!(xGetLastError, vfs_error);
            wrap!(xCurrentTimeInt64, vfs_time64);
            wrap!(xSetSystemCall, vfs_set_call);
            wrap!(xGetSystemCall, vfs_get_call);
            wrap!(xNextSystemCall, vfs_next_call);
            let code = ffi::sqlite3_vfs_register(&mut vfs.base, 1);
            if code != ffi::SQLITE_OK {
                return Err(format!("注册基准 VFS 失败：{code}"));
            }
            Ok(Self {
                vfs,
                _name: name,
                _lock: lock,
            })
        }
    }

    pub fn sample(&self) -> WriteSample {
        let c = &self.vfs.counters;
        WriteSample {
            db_bytes: c.db.load(Ordering::Relaxed),
            wal_bytes: c.wal.load(Ordering::Relaxed),
            other_sqlite_file_bytes: c.other.load(Ordering::Relaxed),
            successful_calls: c.writes.load(Ordering::Relaxed),
            failed_calls: c.failed_writes.load(Ordering::Relaxed),
        }
    }
}

impl Drop for WriteVfs {
    fn drop(&mut self) {
        // SAFETY: 基准 scope 内先关闭连接。每个仍打开的文件另持 Arc，回调均为静态函数。
        unsafe {
            ffi::sqlite3_vfs_register(self.vfs.original, 1);
            ffi::sqlite3_vfs_unregister(&mut self.vfs.base);
        }
    }
}

fn normalize_path(path: &str) -> String {
    let path = path.replace('\\', "/");
    let path = path.strip_prefix("//?/").unwrap_or(&path);
    if cfg!(windows) {
        path.to_lowercase()
    } else {
        path.to_owned()
    }
}

// 所有 VFS 回调都传原 VFS 指针；不依赖宿主回调对 pAppData 或扩展布局的假设。
macro_rules! delegate_vfs {
    ($name:ident, $field:ident, ($($arg:ident: $ty:ty),*) -> $ret:ty) => {
        unsafe extern "C" fn $name(vfs: *mut ffi::sqlite3_vfs, $($arg: $ty),*) -> $ret {
            let original = (*(vfs as *const ProxyVfs)).original;
            ((*original).$field.unwrap())(original, $($arg),*)
        }
    };
}
delegate_vfs!(vfs_delete, xDelete, (name: *const c_char, sync_dir: c_int) -> c_int);
delegate_vfs!(vfs_access, xAccess, (name: *const c_char, flags: c_int, out: *mut c_int) -> c_int);
delegate_vfs!(vfs_fullpath, xFullPathname, (name: *const c_char, n: c_int, out: *mut c_char) -> c_int);
delegate_vfs!(vfs_dl_open, xDlOpen, (name: *const c_char) -> *mut c_void);
delegate_vfs!(vfs_dl_error, xDlError, (n: c_int, out: *mut c_char) -> ());
type DlSymbol = Option<unsafe extern "C" fn(*mut ffi::sqlite3_vfs, *mut c_void, *const c_char)>;
delegate_vfs!(vfs_dl_sym, xDlSym, (handle: *mut c_void, name: *const c_char) -> DlSymbol);
delegate_vfs!(vfs_dl_close, xDlClose, (handle: *mut c_void) -> ());
delegate_vfs!(vfs_random, xRandomness, (n: c_int, out: *mut c_char) -> c_int);
delegate_vfs!(vfs_sleep, xSleep, (micros: c_int) -> c_int);
delegate_vfs!(vfs_time, xCurrentTime, (out: *mut f64) -> c_int);
delegate_vfs!(vfs_error, xGetLastError, (n: c_int, out: *mut c_char) -> c_int);
delegate_vfs!(vfs_time64, xCurrentTimeInt64, (out: *mut i64) -> c_int);
delegate_vfs!(vfs_set_call, xSetSystemCall, (name: *const c_char, callback: ffi::sqlite3_syscall_ptr) -> c_int);
delegate_vfs!(vfs_get_call, xGetSystemCall, (name: *const c_char) -> ffi::sqlite3_syscall_ptr);
delegate_vfs!(vfs_next_call, xNextSystemCall, (name: *const c_char) -> *const c_char);

// SAFETY: xOpen 的 SQLite 分配缓冲区按 szOsFile 包含代理头及原始文件；原始 VFS 拥有后半段。
unsafe extern "C" fn open(
    vfs: *mut ffi::sqlite3_vfs,
    name: *const c_char,
    file: *mut ffi::sqlite3_file,
    flags: c_int,
    out: *mut c_int,
) -> c_int {
    let proxy = &*(vfs as *const ProxyVfs);
    let inner = (file as *mut u8).add(std::mem::size_of::<ProxyFile>()) as *mut ffi::sqlite3_file;
    (*file).pMethods = std::ptr::null();
    (*inner).pMethods = std::ptr::null();
    let code = ((*proxy.original).xOpen.unwrap())(proxy.original, name, inner, flags, out);
    if (*inner).pMethods.is_null() {
        return code;
    }
    let counted = !name.is_null() && {
        let path = normalize_path(&CStr::from_ptr(name).to_string_lossy());
        path.strip_prefix(&proxy.root)
            .is_some_and(|suffix| suffix.starts_with('/'))
    };
    let counters = if counted {
        Arc::into_raw(proxy.counters.clone())
    } else {
        std::ptr::null()
    };
    std::ptr::write(
        file as *mut ProxyFile,
        ProxyFile {
            base: ffi::sqlite3_file { pMethods: &METHODS },
            inner,
            counters,
            kind: flags,
        },
    );
    code
}

unsafe fn inner(file: *mut ffi::sqlite3_file) -> *mut ffi::sqlite3_file {
    (*(file as *mut ProxyFile)).inner
}
unsafe fn methods(file: *mut ffi::sqlite3_file) -> &'static ffi::sqlite3_io_methods {
    &*(*inner(file)).pMethods
}

unsafe extern "C" fn close(file: *mut ffi::sqlite3_file) -> c_int {
    let proxy = &mut *(file as *mut ProxyFile);
    let code = ((*(*proxy.inner).pMethods).xClose.unwrap())(proxy.inner);
    if !proxy.counters.is_null() {
        drop(Arc::from_raw(proxy.counters));
    }
    proxy.base.pMethods = std::ptr::null();
    code
}
unsafe extern "C" fn read(
    file: *mut ffi::sqlite3_file,
    buf: *mut c_void,
    n: c_int,
    offset: i64,
) -> c_int {
    (methods(file).xRead.unwrap())(inner(file), buf, n, offset)
}
unsafe extern "C" fn write(
    file: *mut ffi::sqlite3_file,
    buf: *const c_void,
    n: c_int,
    offset: i64,
) -> c_int {
    let code = (methods(file).xWrite.unwrap())(inner(file), buf, n, offset);
    let proxy = &*(file as *mut ProxyFile);
    if let Some(c) = proxy.counters.as_ref() {
        if code == ffi::SQLITE_OK {
            let counter = if proxy.kind & ffi::SQLITE_OPEN_MAIN_DB != 0 {
                &c.db
            } else if proxy.kind & ffi::SQLITE_OPEN_WAL != 0 {
                &c.wal
            } else {
                &c.other
            };
            counter.fetch_add(n.max(0) as u64, Ordering::Relaxed);
            c.writes.fetch_add(1, Ordering::Relaxed);
        } else {
            c.failed_writes.fetch_add(1, Ordering::Relaxed);
        }
    }
    code
}
unsafe extern "C" fn truncate(file: *mut ffi::sqlite3_file, n: i64) -> c_int {
    (methods(file).xTruncate.unwrap())(inner(file), n)
}
unsafe extern "C" fn sync(file: *mut ffi::sqlite3_file, flags: c_int) -> c_int {
    (methods(file).xSync.unwrap())(inner(file), flags)
}
unsafe extern "C" fn size(file: *mut ffi::sqlite3_file, n: *mut i64) -> c_int {
    (methods(file).xFileSize.unwrap())(inner(file), n)
}
unsafe extern "C" fn lock(file: *mut ffi::sqlite3_file, kind: c_int) -> c_int {
    (methods(file).xLock.unwrap())(inner(file), kind)
}
unsafe extern "C" fn unlock(file: *mut ffi::sqlite3_file, kind: c_int) -> c_int {
    (methods(file).xUnlock.unwrap())(inner(file), kind)
}
unsafe extern "C" fn reserved(file: *mut ffi::sqlite3_file, out: *mut c_int) -> c_int {
    (methods(file).xCheckReservedLock.unwrap())(inner(file), out)
}
unsafe extern "C" fn control(file: *mut ffi::sqlite3_file, op: c_int, arg: *mut c_void) -> c_int {
    (methods(file).xFileControl.unwrap())(inner(file), op, arg)
}
unsafe extern "C" fn sector(file: *mut ffi::sqlite3_file) -> c_int {
    (methods(file).xSectorSize.unwrap())(inner(file))
}
unsafe extern "C" fn device(file: *mut ffi::sqlite3_file) -> c_int {
    (methods(file).xDeviceCharacteristics.unwrap())(inner(file))
}
unsafe extern "C" fn shm_map(
    file: *mut ffi::sqlite3_file,
    page: c_int,
    bytes: c_int,
    extend: c_int,
    out: *mut *mut c_void,
) -> c_int {
    methods(file).xShmMap.map_or(ffi::SQLITE_IOERR_SHMMAP, |f| {
        f(inner(file), page, bytes, extend, out)
    })
}
unsafe extern "C" fn shm_lock(
    file: *mut ffi::sqlite3_file,
    offset: c_int,
    n: c_int,
    flags: c_int,
) -> c_int {
    methods(file)
        .xShmLock
        .map_or(ffi::SQLITE_IOERR_SHMLOCK, |f| {
            f(inner(file), offset, n, flags)
        })
}
unsafe extern "C" fn shm_barrier(file: *mut ffi::sqlite3_file) {
    if let Some(f) = methods(file).xShmBarrier {
        f(inner(file));
    }
}
unsafe extern "C" fn shm_unmap(file: *mut ffi::sqlite3_file, delete: c_int) -> c_int {
    methods(file)
        .xShmUnmap
        .map_or(ffi::SQLITE_OK, |f| f(inner(file), delete))
}
unsafe extern "C" fn fetch(
    file: *mut ffi::sqlite3_file,
    offset: i64,
    n: c_int,
    out: *mut *mut c_void,
) -> c_int {
    match methods(file).xFetch {
        Some(f) => f(inner(file), offset, n, out),
        None => {
            *out = std::ptr::null_mut();
            ffi::SQLITE_OK
        }
    }
}
unsafe extern "C" fn unfetch(file: *mut ffi::sqlite3_file, offset: i64, ptr: *mut c_void) -> c_int {
    methods(file)
        .xUnfetch
        .map_or(ffi::SQLITE_OK, |f| f(inner(file), offset, ptr))
}

static METHODS: ffi::sqlite3_io_methods = ffi::sqlite3_io_methods {
    iVersion: 3,
    xClose: Some(close),
    xRead: Some(read),
    xWrite: Some(write),
    xTruncate: Some(truncate),
    xSync: Some(sync),
    xFileSize: Some(size),
    xLock: Some(lock),
    xUnlock: Some(unlock),
    xCheckReservedLock: Some(reserved),
    xFileControl: Some(control),
    xSectorSize: Some(sector),
    xDeviceCharacteristics: Some(device),
    xShmMap: Some(shm_map),
    xShmLock: Some(shm_lock),
    xShmBarrier: Some(shm_barrier),
    xShmUnmap: Some(shm_unmap),
    xFetch: Some(fetch),
    xUnfetch: Some(unfetch),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_forwards_buffer_offset_and_error_without_counting_failed_bytes() {
        #[repr(C)]
        struct FakeFile {
            base: ffi::sqlite3_file,
            response: c_int,
            n: c_int,
            offset: i64,
            first_byte: u8,
        }
        unsafe extern "C" fn fake_write(
            file: *mut ffi::sqlite3_file,
            buffer: *const c_void,
            n: c_int,
            offset: i64,
        ) -> c_int {
            let f = &mut *(file as *mut FakeFile);
            f.n = n;
            f.offset = offset;
            f.first_byte = *(buffer as *const u8);
            f.response
        }
        // SAFETY: 局部假文件及回调在所有直接调用期间存活；没有注册进程 VFS。
        unsafe {
            let mut callbacks = std::ptr::read(&METHODS);
            callbacks.xWrite = Some(fake_write);
            let mut real = FakeFile {
                base: ffi::sqlite3_file {
                    pMethods: &callbacks,
                },
                response: ffi::SQLITE_IOERR_WRITE,
                n: 0,
                offset: 0,
                first_byte: 0,
            };
            let counters = Arc::new(Counters::default());
            let mut proxy = ProxyFile {
                base: ffi::sqlite3_file { pMethods: &METHODS },
                inner: &mut real.base,
                counters: Arc::as_ptr(&counters),
                kind: ffi::SQLITE_OPEN_WAL,
            };
            let bytes = [42_u8; 14];
            assert_eq!(
                write(&mut proxy.base, bytes.as_ptr().cast(), 14, 901),
                ffi::SQLITE_IOERR_WRITE
            );
            assert_eq!((real.n, real.offset, real.first_byte), (14, 901, 42));
            assert_eq!(counters.wal.load(Ordering::Relaxed), 0);
            assert_eq!(counters.failed_writes.load(Ordering::Relaxed), 1);
            real.response = ffi::SQLITE_OK;
            assert_eq!(
                write(&mut proxy.base, bytes.as_ptr().cast(), 14, 901),
                real.response
            );
            assert_eq!(counters.wal.load(Ordering::Relaxed), 14);
            assert_eq!(counters.writes.load(Ordering::Relaxed), 1);
        }
    }

    #[test]
    fn counts_only_isolated_files_and_preserves_wal_and_errors() {
        if crate::bench::run_isolated_test(
            "bench::write_vfs::tests::counts_only_isolated_files_and_preserves_wal_and_errors",
        ) {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let vfs = WriteVfs::install(dir.path()).unwrap();
        let db =
            crate::storage::StorageCoordinator::open(&dir.path().join("monitor.sqlite3")).unwrap();
        let before = vfs.sample();
        db.connection()
            .execute_batch("create table probe(value integer unique); insert into probe values(42)")
            .unwrap();
        let written = vfs.sample().since(before);
        assert!(written.wal_bytes > 0);
        assert_eq!(
            db.connection()
                .query_row("pragma synchronous", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert!(db
            .connection()
            .execute("insert into probe values(42)", [])
            .is_err());
        let reader = rusqlite::Connection::open(db.path()).unwrap();
        assert_eq!(
            reader
                .query_row("select value from probe", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            42
        );
        let before_other = vfs.sample();
        let outside = rusqlite::Connection::open(other.path().join("other.sqlite3")).unwrap();
        outside
            .execute_batch("create table other(value); insert into other values(1)")
            .unwrap();
        assert_eq!(vfs.sample().since(before_other).successful_calls, 0);
        drop(reader);
        db.connection()
            .execute_batch("pragma wal_checkpoint(truncate)")
            .unwrap();
        assert!(vfs.sample().since(before_other).db_bytes > 0);
        assert_eq!(
            db.connection()
                .query_row("pragma integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    }
}
