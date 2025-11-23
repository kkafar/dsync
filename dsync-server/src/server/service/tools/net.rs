pub mod addr_discovery;

pub fn ipv4_into_connection_addr(ipaddr: impl AsRef<str>, port: u16) -> String {
    let ip_addr = ipaddr.as_ref();
    "http://".to_owned() + ip_addr + ":" + port.to_string().as_str()
}
