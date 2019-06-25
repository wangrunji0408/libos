pub use sgx_trts::libc;
pub use sgx_trts::libc::off_t;
pub use sgx_types::*;

//pub use {elf_helper, errno, file, file_table, fs, mm, process, syscall, vma, };

pub use alloc::boxed::Box;
pub use alloc::collections::{BTreeMap, VecDeque};
pub use alloc::prelude::ToOwned;
pub use alloc::rc::Rc;
pub use alloc::string::String;
pub use alloc::vec::Vec;
pub use core::cell::{Cell, RefCell};
pub use core::cmp::{max, min};
pub use core::cmp::{Ordering, PartialOrd};
pub use core::fmt::{Debug, Display};
pub use core::iter::Iterator;
pub use core::marker::{Send, Sync};
pub use core::result::Result;
pub use std::io::{Read, Seek, SeekFrom, Write};
pub use std::sync::{
    Arc, SgxMutex, SgxMutexGuard, SgxRwLock, SgxRwLockReadGuard, SgxRwLockWriteGuard,
};

pub use errno::Errno;
pub use errno::Errno::*;
pub use errno::Error;

macro_rules! debug_trace {
    () => {
        println!("> Line = {}, File = {}", line!(), file!())
    };
}

macro_rules! errno {
    ($errno: ident, $msg: expr) => {{
        error!(
            "ERROR: {} ({}, line {} in file {})",
            $errno,
            $msg,
            line!(),
            file!()
        );
        Err(Error::new($errno, $msg))
    }};
}

// return Err(errno) if libc return -1
macro_rules! try_libc {
    ($ret: expr) => {{
        let ret = unsafe { $ret };
        if ret == -1 {
            let errno = unsafe { libc::errno() };
            // println will cause libc ocall and overwrite errno
            error!(
                "ERROR from libc: {} (line {} in file {})",
                errno,
                line!(),
                file!()
            );
            return Err(Error::new(Errno::from_errno(errno), "libc error"));
        }
        ret
    }};
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + (align - 1)) / align * align
}

pub fn align_down(addr: usize, align: usize) -> usize {
    addr & !(align - 1)
}

pub fn unbox<T>(value: Box<T>) -> T {
    *value
}
