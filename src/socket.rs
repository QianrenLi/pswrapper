use std::net::UdpSocket;

#[cfg(unix)]
pub fn create_udp_socket(tos: u8, tx_ipaddr: String) -> Option<UdpSocket> {
    use std::os::unix::io::AsRawFd;

    let sock = UdpSocket::bind(format!("{}:0", tx_ipaddr)).ok()?;
    print!("tx_ipaddr: {}\n", tx_ipaddr);
    let res = unsafe{
        let fd = sock.as_raw_fd();
        let value = &(tos as i32) as *const libc::c_int as *const libc::c_void;
        let option_len = std::mem::size_of::<libc::c_int>() as u32;
        libc::setsockopt(fd, libc::IPPROTO_IP, libc::IP_TOS, value, option_len)
    };
    
    if res == 0 { Some(sock) } else { None }
}