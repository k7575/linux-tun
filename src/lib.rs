use std::fs::File;
use std::io::{Error, Result};
use std::os::unix::io::{AsRawFd, RawFd};

use libc::{
    AF_INET, IFF_NO_PI, IFF_RUNNING, IFF_TUN, IFF_UP, IFNAMSIZ, SIOCGIFFLAGS, SIOCSIFADDR,
    SIOCSIFFLAGS, SIOCSIFNETMASK, SOCK_DGRAM, close, ioctl, socket,
};

// fix it for all platforms
const TUNSETIFF: u64 = 0x400454ca;

#[repr(C)]
#[derive(Clone, Copy)]
struct SockaddrIn {
    sin_family: u16,
    sin_port: u16,
    sin_addr: [u8; 4],
    sin_zero: [u8; 8],
}

#[repr(C)]
union IfreqUnion {
    ifr_addr: SockaddrIn,
    ifr_flags: i16,
}

#[repr(C)]
struct Ifreq {
    ifr_name: [u8; IFNAMSIZ],
    ifr_ifru: IfreqUnion,
}

struct SafeSocket(RawFd);

impl SafeSocket {
    fn new() -> Result<Self> {
        let fd = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
        if fd < 0 {
            return Err(Error::last_os_error());
        }
        Ok(SafeSocket(fd))
    }
}

impl Drop for SafeSocket {
    fn drop(&mut self) {
        unsafe { close(self.0) };
    }
}

fn set_ifreq_name(ifr: &mut Ifreq, name: &str) {
    let bytes = name.as_bytes();
    let len = bytes.len().min(IFNAMSIZ - 1);
    ifr.ifr_name[..len].copy_from_slice(&bytes[..len]);
    ifr.ifr_name[len] = 0;
}

pub fn create_tun(name: &str) -> Result<File> {
    let file = File::options()
        .read(true)
        .write(true)
        .open("/dev/net/tun")?;

    let mut ifr: Ifreq = unsafe { std::mem::zeroed() };
    set_ifreq_name(&mut ifr, name);

    unsafe {
        ifr.ifr_ifru.ifr_flags = (IFF_TUN | IFF_NO_PI) as i16;
        if ioctl(file.as_raw_fd(), TUNSETIFF as _, &ifr) < 0 {
            return Err(Error::last_os_error());
        }
    }

    Ok(file)
}

pub fn setup_interface(name: &str, ip: [u8; 4], mask: [u8; 4]) -> Result<()> {
    let sock = SafeSocket::new()?;
    let mut ifr: Ifreq = unsafe { std::mem::zeroed() };
    set_ifreq_name(&mut ifr, name);

    let addr = SockaddrIn {
        sin_family: AF_INET as u16,
        sin_port: 0,
        sin_addr: ip,
        sin_zero: [0; 8],
    };
    ifr.ifr_ifru.ifr_addr = addr;
    if unsafe { ioctl(sock.0, SIOCSIFADDR as _, &ifr) } < 0 {
        return Err(Error::last_os_error());
    }

    let netmask = SockaddrIn {
        sin_family: AF_INET as u16,
        sin_port: 0,
        sin_addr: mask,
        sin_zero: [0; 8],
    };
    ifr.ifr_ifru.ifr_addr = netmask;
    if unsafe { ioctl(sock.0, SIOCSIFNETMASK as _, &ifr) } < 0 {
        return Err(Error::last_os_error());
    }

    if unsafe { ioctl(sock.0, SIOCGIFFLAGS as _, &ifr) } < 0 {
        return Err(Error::last_os_error());
    }

    unsafe {
        ifr.ifr_ifru.ifr_flags |= (IFF_UP | IFF_RUNNING) as i16;
        if ioctl(sock.0, SIOCSIFFLAGS as _, &ifr) < 0 {
            return Err(Error::last_os_error());
        }
    }

    Ok(())
}

pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    for chunk in data.chunks_exact(2) {
        sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }
    if data.len() % 2 != 0 {
        sum += (data[data.len() - 1] as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}
