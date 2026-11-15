use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::path::PathBuf;

use atomic_delegate_macros::atomic_delegates;

use crate::freenet_node;

#[derive(Debug)]
pub struct EmbeddedNode {
    pub node_dir: freenet_node::NodeDir,
    pub websocket_port: u16,
}

#[atomic_delegates]
impl EmbeddedNode {
    pub async fn start(kept_node_dir: Option<PathBuf>) -> Result<Self, freenet_node::Error> {}
}

impl EmbeddedNode {
    pub(crate) fn free_udp_port() -> Result<u16, freenet_node::Error> {
        let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
            .map_err(freenet_node::Error::Bind)?;
        let address = socket.local_addr().map_err(freenet_node::Error::Bind)?;
        Ok(address.port())
    }
}

#[cfg(test)]
mod tests {
    use crate::freenet_node;

    #[test]
    fn test_usage() {
        let port = freenet_node::EmbeddedNode::free_udp_port().expect("port");
        assert!(port > 0);
        let _ = freenet_node::EmbeddedNode::start;
    }
}
