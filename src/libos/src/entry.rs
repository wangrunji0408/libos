use super::*;
use std::path::Path;
use util::mem_util::from_untrusted::*;

#[no_mangle]
pub extern "C" fn libos_boot(path_buf: *const c_char, argv: *const *const c_char) -> i32 {
    util::log::init();
    let (path, args) = match parse_arguments(path_buf, argv) {
        Ok(path_and_args) => path_and_args,
        Err(_) => {
            return EXIT_STATUS_INTERNAL_ERROR;
        }
    };

    match do_boot(&path, &args) {
        Ok(()) => 0,
        Err(err) => EXIT_STATUS_INTERNAL_ERROR,
    }
}

#[no_mangle]
pub extern "C" fn libos_run() -> i32 {
    match do_run() {
        Ok(exit_status) => exit_status,
        Err(err) => EXIT_STATUS_INTERNAL_ERROR,
    }
}

#[no_mangle]
pub extern "C" fn dummy_ecall() -> i32 {
    0
}
// Use 127 as a special value to indicate internal error from libos, not from
// user programs, although it is completely ok for a user program to return 127.
const EXIT_STATUS_INTERNAL_ERROR: i32 = 127;

fn parse_arguments(
    path_buf: *const c_char,
    argv: *const *const c_char,
) -> Result<(String, Vec<String>), Error> {
    let path_string = clone_cstring_safely(path_buf)?;
    let program_cstring = {
        let program_osstr = Path::new(&path_string)
            .file_name()
            .ok_or_else(|| Error::new(Errno::EINVAL, "Invalid path"))?;
        let program_str = program_osstr
            .to_str()
            .ok_or_else(|| Error::new(Errno::EINVAL, "Invalid path"))?;
        String::from(program_str)
    };

    let mut args = clone_cstrings_safely(argv)?;
    args.insert(0, program_cstring);
    Ok((path_string, args))
}

// TODO: make sure do_boot can only be called once
fn do_boot(path_str: &str, argv: &Vec<String>) -> Result<(), Error> {
    //    info!("boot: path: {:?}, argv: {:?}", path_str, argv);
    util::mpx_util::mpx_enable()?;

    let envp = alloc::vec::Vec::new();
    let file_actions = Vec::new();
    let parent = &process::IDLE_PROCESS;
    process::do_spawn(&path_str, argv, &envp, &file_actions, parent)?;

    Ok(())
}

// TODO: make sure do_run() cannot be called after do_boot()
fn do_run() -> Result<i32, Error> {
    let exit_status = process::run_task()?;

    // sync file system
    // TODO: only sync when all processes exit
    use rcore_fs::vfs::FileSystem;
    crate::fs::ROOT_INODE.fs().sync()?;

    Ok(exit_status)
}
