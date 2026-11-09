#[must_use]
pub fn discover_lan_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    if addr.ip().is_unspecified() || addr.ip().is_loopback() {
        return None;
    }
    Some(addr.ip().to_string())
}

#[cfg(test)]
mod tests {
    use super::discover_lan_ip;

    #[test]
    fn test_usage() {
        let found = discover_lan_ip();
        if let Some(ip) = found {
            let parsed: std::net::IpAddr = ip.parse().expect("numeric ip");
            assert!(!parsed.is_unspecified());
            assert!(!parsed.is_loopback());
        }
    }
}
