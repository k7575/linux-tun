use std::fs::File;
use std::io::{Error, Result};
use std::mem;
use std::os::unix::io::AsRawFd;

const TUNSETIFF: u64 = 0x400454ca;
const IFF_TUN: i16 = 0x0001;
const IFF_NO_PI: i16 = 0x1000;

const SIOCSIFADDR: u64 = 0x8916;
const SIOCSIFFLAGS: u64 = 0x8914;
const IFF_UP: i16 = 0x1;
const IFF_RUNNING: i16 = 0x40;
const SIOCSIFNETMASK: u64 = 0x891c;

#[repr(C)]
struct Ifreq {
    ifr_name: [u8; 16],
    ifr_flags: i16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SockaddrIn {
    sin_family: u16,
    sin_port: u16,
    sin_addr: [u8; 4],
    sin_zero: [u8; 8],
}

#[repr(C)]
union IfreqAddr {
    ifr_addr: SockaddrIn,
    ifr_flags: i16,
}

#[repr(C)]
struct IfreqOp {
    ifr_name: [u8; 16],
    ifr_ifru: IfreqAddr,
}

pub fn create_tun(name: &str) -> Result<File> {
    let file = File::options()
        .read(true)
        .write(true)
        .open("/dev/net/tun")?;

    let mut ifr: Ifreq = unsafe { mem::zeroed() };

    let bytes = name.as_bytes();
    let len = bytes.len().min(15);
    ifr.ifr_name[..len].copy_from_slice(&bytes[..len]);

    ifr.ifr_flags = IFF_TUN | IFF_NO_PI;

    let fd = file.as_raw_fd();
    let ret = unsafe { libc::ioctl(fd, TUNSETIFF as _, &ifr) };

    if ret < 0 {
        return Err(Error::last_os_error());
    }

    Ok(file)
}

pub fn setup_interface(name: &str, ip: [u8; 4], mask: [u8; 4]) -> std::io::Result<()> {
    unsafe {
        let sock = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
        if sock < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut ifr: IfreqOp = std::mem::zeroed();
        let bytes = name.as_bytes();
        ifr.ifr_name[..bytes.len().min(15)].copy_from_slice(&bytes[..bytes.len().min(15)]);

        let addr = SockaddrIn {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: ip,
            sin_zero: [0; 8],
        };
        ifr.ifr_ifru.ifr_addr = addr;
        if libc::ioctl(sock, SIOCSIFADDR, &ifr) < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let netmask = SockaddrIn {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: mask,
            sin_zero: [0; 8],
        };
        ifr.ifr_ifru.ifr_addr = netmask;
        if libc::ioctl(sock, SIOCSIFNETMASK, &ifr) < 0 {
            return Err(std::io::Error::last_os_error());
        }

        if libc::ioctl(sock, libc::SIOCGIFFLAGS as _, &ifr) < 0 {
            return Err(std::io::Error::last_os_error());
        }
        ifr.ifr_ifru.ifr_flags |= IFF_UP | IFF_RUNNING;
        if libc::ioctl(sock, SIOCSIFFLAGS, &ifr) < 0 {
            return Err(std::io::Error::last_os_error());
        }

        libc::close(sock);
    }
    Ok(())
}

pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    for chunk in data.chunks_exact(2) {
        sum += u16::from_be_bytes([chunk[0], chunk[1]]) as u32;
    }
    if data.len() % 2 != 0 {
        sum += (*data.last().unwrap() as u32) << 8;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !sum as u16
}
