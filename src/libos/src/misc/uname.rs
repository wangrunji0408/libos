use super::*;

/// A sample of `struct utsname`
/// ```
///   sysname = Linux
///   nodename = tian-nuc
///   release = 4.15.0-42-generic
///   version = #45~16.04.1-Ubuntu SMP Mon Nov 19 13:02:27 UTC 2018
///   machine = x86_64
///   domainname = (none)
/// ```
///
/// By the way, UTS stands for UNIX Timesharing System.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct utsname_t {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
    domainname: [u8; 65],
}

pub fn do_uname(name: &mut utsname_t) -> Result<(), Error> {
    copy_from_cstr_to_u8_array(SYSNAME, &mut name.sysname);
    copy_from_cstr_to_u8_array(NODENAME, &mut name.nodename);
    copy_from_cstr_to_u8_array(RELEASE, &mut name.release);
    copy_from_cstr_to_u8_array(VERSION, &mut name.version);
    copy_from_cstr_to_u8_array(MACHINE, &mut name.machine);
    copy_from_cstr_to_u8_array(DOMAINNAME, &mut name.domainname);
    Ok(())
}

const SYSNAME: &str = "Occlum";
const NODENAME: &str = "occlum-node";
const RELEASE: &str = "0.1";
const VERSION: &str = "0.1";
const MACHINE: &str = "x86-64";
const DOMAINNAME: &str = "";

fn copy_from_cstr_to_u8_array(src: &str, dst: &mut [u8]) {
    let len = min(dst.len() - 1, src.as_bytes().len());
    dst[..len].copy_from_slice(&src.as_bytes()[..len]);
    dst[len] = 0;
}
