# Rust TUN Interface Helper

A lightweight Rust utility for creating and configuring virtual TUN (Network Tunnel) interfaces on Linux using raw `libc` ioctls.

## Features

- **TUN Creation**: Opens `/dev/net/tun` and initializes a new virtual interface.
- **Interface Configuration**: Sets IP addresses and netmasks directly via socket ioctls.
- **State Management**: Automatically brings the interface "UP" and "RUNNING".
- **Utilities**: Includes a standard Internet Checksum implementation for IP/TCP/UDP headers.

## Prerequisites

- **OS**: Linux (uses specific Unix ioctls and `/dev/net/tun`).
- **Permissions**: Running code that creates network interfaces typically requires `CAP_NET_ADMIN` or `sudo`.

## Usage

Add `libc` to your `Cargo.toml`:
```toml
[dependencies]
libc = "0.2"
```

### Basic Example

```rust
fn main() -> std::io::Result<()> {
    let iface_name = "tun0";
    
    // 1. Create the TUN device
    let tun_file = create_tun(iface_name)?;
    
    // 2. Configure IP and Mask (e.g., 10.0.0.1/255.255.255.0)
    setup_interface(iface_name, [10, 0, 0, 1], [255, 255, 255, 0])?;
    
    println!("Interface {} is up!", iface_name);
    Ok(())
}
```

## How it Works

1.  **`create_tun`**: Uses `TUNSETIFF` to register a new device with the kernel.
2.  **`setup_interface`**: Uses a temporary `AF_INET` socket to send `SIOCSIFADDR` (IP), `SIOCSIFNETMASK` (Mask), and `SIOCSIFFLAGS` (Up/Running) commands.
3.  **`SafeSocket`**: A small wrapper to ensure the control socket is closed automatically, preventing file descriptor leaks.
4.  **`checksum`**: Computes the 1s-complement sum required for network protocol headers.

## Security Warning

This crate uses `unsafe` blocks to interface with C system calls. Ensure that the interface names and parameters passed to these functions are validated to avoid undefined behavior.
