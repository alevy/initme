use libc::*;
use nix::sys::socket::{self, InetAddr, IpAddr, SockAddr};

#[repr(C)]
#[derive(Copy, Clone)]
struct ifreq_addr {
    ifr_name: [u8; libc::IFNAMSIZ],
    ifr_addr: libc::sockaddr,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ifreq_flags {
    ifr_name: [u8; libc::IFNAMSIZ],
    ifr_flags: libc::c_short,
}

pub(crate) fn setup(dev: &str, ip: IpAddr, broadcast: IpAddr, gateway: IpAddr) -> nix::Result<()> {
    let sockfd = socket::socket(
        socket::AddressFamily::Inet,
        socket::SockType::Datagram,
        socket::SockFlag::empty(),
        None,
    )?;

    let mut device_name = [0; IFNAMSIZ];
    let dev = dev.as_bytes();
    device_name[0..dev.len()].copy_from_slice(dev);
    device_name[dev.len()] = 0;

    let myaddr = SockAddr::new_inet(InetAddr::new(ip, 0));
    let mynetmask = SockAddr::new_inet(InetAddr::new(IpAddr::new_v4(255, 255, 255, 0), 0));
    let mybroadcast = SockAddr::new_inet(InetAddr::new(broadcast, 0));
    let mygateway = SockAddr::new_inet(InetAddr::new(gateway, 0));
    let mydst = SockAddr::new_inet(InetAddr::new(IpAddr::new_v4(0, 0, 0, 0), 0));

    // set ip address
    {
        let ifr = ifreq_addr {
            ifr_name: device_name,
            ifr_addr: *myaddr.as_ffi_pair().0,
        };
        unsafe {
            ioctl(sockfd, SIOCSIFADDR as i32, &ifr);
        }
    }

    // set broadcast address
    {
        let ifr = ifreq_addr {
            ifr_name: device_name,
            ifr_addr: *mybroadcast.as_ffi_pair().0,
        };
        unsafe {
            ioctl(sockfd, SIOCSIFBRDADDR as i32, &ifr);
        }
    }

    // set netmask
    {
        let ifr = ifreq_addr {
            ifr_name: device_name,
            ifr_addr: *mynetmask.as_ffi_pair().0,
        };
        unsafe {
            ioctl(sockfd, SIOCSIFNETMASK as i32, &ifr);
        }
    }

    // ifconfig eth0 up
    {
        let ifr = ifreq_flags {
            ifr_name: device_name,
            ifr_flags: IFF_UP as i16,
        };
        unsafe {
            ioctl(sockfd, SIOCSIFFLAGS as i32, &ifr);
        }
    }

    // route add default gw 10.0.1.2
    {
        let rt = rtentry {
            rt_pad1: 0,
            rt_pad2: 0,
            rt_pad3: 0,
            rt_tos: 0,
            rt_class: 0,
            rt_pad4: [0; 3],
            rt_metric: 0,
            rt_mtu: 0,
            rt_window: 0,
            rt_irtt: 0,
            rt_dev: std::ptr::null_mut(),
            rt_dst: *mydst.as_ffi_pair().0,
            rt_gateway: *mygateway.as_ffi_pair().0,
            rt_genmask: *mydst.as_ffi_pair().0,
            rt_flags: RTF_UP | RTF_GATEWAY,
        };
        unsafe {
            ioctl(sockfd, SIOCADDRT as i32, &rt);
        }
    }
    Ok(())
}
