#![allow(unused)]
#![crate_name = "occlum_rs"]
#![crate_type = "staticlib"]
#![no_std]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(range_contains)]
#![feature(core_intrinsics)]
#![feature(core_panic_info)]
#![feature(thread_local)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_trts;
extern crate sgx_tprotected_fs;
extern crate spin;
extern crate xmas_elf;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rcore_fs;
extern crate rcore_fs_sefs;
#[macro_use]
extern crate derive_builder;

use sgx_trts::libc;
use sgx_types::*;

#[macro_use]
mod prelude;
mod entry;
mod errno;
mod fs;
mod misc;
mod process;
mod syscall;
mod time;
mod util;
mod vm;

use prelude::*;

// Export system calls
pub use syscall::*;
// Export ECalls
pub use entry::*;
