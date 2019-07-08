use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use rcore_fs::dev::TimeProvider;
use rcore_fs::vfs::Timespec;
use rcore_fs_sefs::dev::*;
use spin::Mutex;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sgxfs::{remove, OpenOptions, SgxFile};
use std::time::{SystemTime, UNIX_EPOCH};
use sgx_tprotected_fs::{SgxFileStream, SeekFrom as SeekFrom_};
use sgx_trts::c_str::CStr;

pub struct SgxStorage {
    path: PathBuf,
    file_cache: Mutex<BTreeMap<usize, LockedFile>>,
}

impl SgxStorage {
    pub fn new(path: impl AsRef<Path>) -> Self {
        //        assert!(path.as_ref().is_dir());
        SgxStorage {
            path: path.as_ref().to_path_buf(),
            file_cache: Mutex::new(BTreeMap::new()),
        }
    }
    /// Get file by `file_id`.
    /// It lookups cache first, if miss, then call `open_fn` to open one,
    /// and add it to cache before return.
    #[cfg(feature = "sgx_file_cache")]
    fn get(&self, file_id: usize, open_fn: impl FnOnce(&Self) -> LockedFile) -> LockedFile {
        // query cache
        let mut caches = self.file_cache.lock();
        if let Some(locked_file) = caches.get(&file_id) {
            // hit, return
            return locked_file.clone();
        }
        // miss, open one
        let locked_file = open_fn(self);
        // add to cache
        caches.insert(file_id, locked_file.clone());
        locked_file
    }
    /// Get file by `file_id` without cache.
    #[cfg(not(feature = "sgx_file_cache"))]
    fn get(&self, file_id: usize, open_fn: impl FnOnce(&Self) -> LockedFile) -> LockedFile {
        open_fn(self)
    }
}

impl Storage for SgxStorage {
    fn open(&self, file_id: usize) -> DevResult<Box<File>> {
        let locked_file = self.get(file_id, |this| {
            let mut path = this.path.to_path_buf();
            path.push(format!("{}\0", file_id));
            // TODO: key
            let key = [0u8; 16];
            let path = CStr::from_bytes_with_nul(path.to_str().unwrap().as_bytes()).unwrap();
            let mode = CStr::from_bytes_with_nul(b"r+b\0").unwrap();
            let file = SgxFileStream::open(path, mode, &key)
                .expect("failed to open SgxFile");
            LockedFile(Arc::new(Mutex::new(file)))
        });
        Ok(Box::new(locked_file))
    }

    fn create(&self, file_id: usize) -> DevResult<Box<File>> {
        let locked_file = self.get(file_id, |this| {
            let mut path = this.path.to_path_buf();
            path.push(format!("{}", file_id));
            // TODO: key
            let key = [0u8; 16];
            let path = CStr::from_bytes_with_nul(path.to_str().unwrap().as_bytes()).unwrap();
            let mode = CStr::from_bytes_with_nul(b"w+b\0").unwrap();
            let file = SgxFileStream::open(path, mode, &key)
                .expect("failed to open SgxFile");
            LockedFile(Arc::new(Mutex::new(file)))
        });
        Ok(Box::new(locked_file))
    }

    fn remove(&self, file_id: usize) -> DevResult<()> {
        let mut path = self.path.to_path_buf();
        path.push(format!("{}", file_id));
        remove(path).expect("failed to remove SgxFile");
        // remove from cache
        let mut caches = self.file_cache.lock();
        caches.remove(&file_id);
        Ok(())
    }
}

#[derive(Clone)]
pub struct LockedFile(Arc<Mutex<SgxFileStream>>);

// `sgx_tstd::sgxfs::SgxFile` not impl Send ...
unsafe impl Send for LockedFile {}
unsafe impl Sync for LockedFile {}

impl File for LockedFile {
    fn read_at(&self, buf: &mut [u8], offset: usize) -> DevResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let mut file = self.0.lock();
        file.seek(offset as i64, SeekFrom_::Start)
            .expect("failed to seek SgxFile");
        let len = file.read(buf).expect("failed to read SgxFile");
        Ok(len)
    }

    fn write_at(&self, buf: &[u8], offset: usize) -> DevResult<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }
        let mut file = self.0.lock();

        // SgxFile do not support seek a position after the end.
        // So check the size and padding zeros if necessary.
        file.seek(0, SeekFrom_::End).expect("failed to seek SgxFile");
        let file_size = file.tell().expect("failed to tell SgxFile") as usize;
        if file_size < offset {
            static ZEROS: [u8; 0x1000] = [0; 0x1000];
            let mut rest_len = offset - file_size;
            while rest_len != 0 {
                let l = rest_len.min(0x1000);
                let len = file.write(&ZEROS[..l]).expect("failed to write SgxFile");
                rest_len -= len;
            }
        }

        let offset = offset as u64;
        file.seek(offset as i64, SeekFrom_::Start)
            .expect("failed to seek SgxFile");
        let len = file.write(buf).expect("failed to write SgxFile");
        Ok(len)
    }

    fn set_len(&self, len: usize) -> DevResult<()> {
        // NOTE: do nothing ??
        Ok(())
    }

    fn flush(&self) -> DevResult<()> {
        let mut file = self.0.lock();
        file.flush().expect("failed to flush SgxFile");
        Ok(())
    }
}
