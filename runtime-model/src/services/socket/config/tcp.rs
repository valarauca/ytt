use std::net::{SocketAddr};
use std::io::{self};

use socket2::{Socket,Domain,Type};
use serde::{Serialize,Deserialize};
#[allow(unused_imports)] use tracing::{error,warn};

use super::keepalive::{KeepAlive};
use config_crap::{NiceDuration};

#[derive(Clone,PartialEq,Eq,Serialize,Deserialize,Debug)]
pub struct TcpListenerConfig {
    pub address: SocketAddr,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub only_ipv6: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub bind_device: Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub keepalive: Option<KeepAlive>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub no_delay: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub ttl: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub tos: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub max_segment: Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub recv_buffer_size: Option<usize>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub send_buffer_size: Option<usize>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub reuse_port: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub reuse_address: Option<bool>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub read_timeout: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub write_timeout: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub linger: Option<NiceDuration>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub backlog: Option<u16>
}
impl TcpListenerConfig {

    pub fn bind_socket(&self) -> io::Result<std::net::TcpListener> {
        let socket = match &self.address {
            SocketAddr::V4(_) => Socket::new(Domain::IPV4, Type::STREAM, None)?,
            SocketAddr::V6(_) => Socket::new(Domain::IPV6, Type::STREAM, None)?,
        };

        if let Option::Some(only_ipv6) = &self.only_ipv6 {
            if self.address.is_ipv6() {
                socket.set_only_v6(*only_ipv6)?;
            } else if *only_ipv6 {
                warn!("cannot set only ipv6 for a IPv4 listening socket");
            }
        }
        if let Option::Some(interface) = &self.bind_device {
            os_specific_bind_device(&socket, &interface)?;
        }
        if let Option::Some(keepalive) = KeepAlive::build(&self.keepalive) {
            socket.set_tcp_keepalive(&keepalive)?;
        }
        if let Option::Some(no_delay) = &self.no_delay {
            socket.set_tcp_nodelay(*no_delay)?;
        }
        if let Option::Some(ttl) = &self.ttl {
            socket.set_ttl_v4(*ttl)?;
        }
        if let Option::Some(tos) = &self.tos {
            socket.set_tos_v4(*tos)?;
        }
        if let Option::Some(recv_buffer_size) = &self.recv_buffer_size {
            socket.set_recv_buffer_size(*recv_buffer_size)?;
        }
        if let Option::Some(send_buffer_size) = &self.send_buffer_size {
            socket.set_send_buffer_size(*send_buffer_size)?;
        }
        if let Option::Some(reuse_port) = &self.reuse_port {
            os_specific_reuse_port(&socket,reuse_port)?;
        }
        if let Option::Some(reuse_address) = &self.reuse_address {
            socket.set_reuse_address(*reuse_address)?;
        }
        if let Option::Some(read_timeout) = &self.read_timeout {
            socket.set_read_timeout(Some(read_timeout.get_duration()))?;
        }
        if let Option::Some(write_timeout) = &self.write_timeout {
            socket.set_write_timeout(Some(write_timeout.get_duration()))?;
        }
        if let Option::Some(linger) = &self.linger {
            socket.set_linger(Some(linger.get_duration()))?;
        }
        if let Option::Some(maxseg) = &self.max_segment {
            os_specific_set_mss(&socket,maxseg)?;
        }

        socket.set_nonblocking(true)?;
        let addr = socket2::SockAddr::from(self.address.clone());
        socket.connect(&addr)?;
        socket.listen(self.backlog.clone().unwrap_or_else(|| 128) as i32)?;

        Ok(socket.into())
    }
}

#[cfg(any(unix,target_os = "solaris", target_os = "illumos", target_os = "cygwin"))]
fn os_specific_reuse_port(socket: &Socket, arg: &bool) -> io::Result<()> {
    socket.set_reuse_port(*arg)?;
    Ok(())
}
#[cfg(not(any(unix,target_os = "solaris", target_os = "illumos", target_os = "cygwin")))]
fn os_specific_reuse_port(_socket: &Socket, _arg: &bool) -> io::Result<()> {
    warn!("setting port reuse is not support on this platform");
    Ok(())
}

#[cfg(all(unix, not(target_os = "redox")))]
fn os_specific_set_mss(socket: &Socket, arg: &u32) -> io::Result<()> {
    socket.set_tcp_mss(*arg)?;
    Ok(())
}
#[cfg(not(all(unix, not(target_os = "redox"))))]
fn os_specific_set_mss(_socket: &Socket, _arg: &u32) -> io::Result<()> {
    warn!("setting TCP_MAXSEG is not presently supported on non-unix hosts");
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
fn os_specific_bind_device(socket: &Socket, arg: &str) -> io::Result<()> {
    use std::ffi::CString;

    if arg.is_empty() {
        warn!("bind_device name set, but empty");
        return Ok(())
    }

    if !arg.is_ascii() {
        error!("device name '{}' contains non-ascii characters. Documentation suggests this is unsupported.", arg);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "device names must only have asci characters"));
    }
    let device = match CString::new(arg) {
        Err(_) => {
            error!("device name '{}' contains a null character", arg);
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "device names cannot contain nul characters"));
        }
        Ok(device) => device,
    };

    socket.bind_device(Some(device.as_bytes()))
}

#[cfg(not(any(target_os = "android", target_os = "fuchsia", target_os = "linux")))]
fn os_specific_bind_device(_socket: &Socket, _arg: &str) -> io::Result<()> {
    warn!("cannot bind to device on this operating system");
    Ok(())
}
