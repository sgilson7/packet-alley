//! IPv4 addresses and prefixes: parsing, printing, and the subnet test every
//! host and router makes ("is this address on my block?").

/// An IPv4 address as its 32-bit number.
pub type Ip = u32;

/// An address with its prefix length, as written `10.0.1.10/24`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cidr {
    pub ip: Ip,
    pub len: u8,
}

pub fn parse_ip(s: &str) -> Option<Ip> {
    let parts: Vec<&str> = s.trim().split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut out: u32 = 0;
    for p in parts {
        if p.is_empty() || p.len() > 3 || !p.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let v: u32 = p.parse().ok()?;
        if v > 255 {
            return None;
        }
        out = (out << 8) | v;
    }
    Some(out)
}

pub fn parse_cidr(s: &str) -> Option<Cidr> {
    let (a, l) = s.trim().split_once('/')?;
    let len: u8 = l.parse().ok()?;
    if len > 32 {
        return None;
    }
    Some(Cidr {
        ip: parse_ip(a)?,
        len,
    })
}

pub fn show_ip(ip: Ip) -> String {
    format!(
        "{}.{}.{}.{}",
        ip >> 24,
        (ip >> 16) & 255,
        (ip >> 8) & 255,
        ip & 255
    )
}

pub fn mask(len: u8) -> u32 {
    if len == 0 {
        0
    } else {
        u32::MAX << (32 - len as u32)
    }
}

impl Cidr {
    /// The network this address is on: `10.0.1.10/24` gives `10.0.1.0/24`.
    pub fn network(self) -> Cidr {
        Cidr {
            ip: self.ip & mask(self.len),
            len: self.len,
        }
    }

    pub fn contains(self, ip: Ip) -> bool {
        (ip & mask(self.len)) == (self.ip & mask(self.len))
    }

    pub fn show(self) -> String {
        format!("{}/{}", show_ip(self.ip), self.len)
    }
}

/// A MAC address is shown as six pairs; the simulation numbers them.
pub fn show_mac(mac: u32) -> String {
    if mac == BROADCAST_MAC {
        return "ff:ff:ff:ff:ff:ff".to_string();
    }
    format!("02:00:00:00:{:02x}:{:02x}", (mac >> 8) & 255, mac & 255)
}

pub const BROADCAST_MAC: u32 = u32::MAX;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_round_trip() {
        assert_eq!(parse_ip("10.0.1.10"), Some(0x0a00010a));
        assert_eq!(show_ip(0x0a00010a), "10.0.1.10");
        assert_eq!(parse_ip("10.0.1"), None);
        assert_eq!(parse_ip("10.0.1.256"), None);
        assert_eq!(parse_ip("10.0.-1.2"), None);
        assert_eq!(
            parse_cidr("10.0.1.10/24").map(|c| c.network().show()),
            Some("10.0.1.0/24".into())
        );
        assert_eq!(parse_cidr("10.0.1.10/33"), None);
    }

    #[test]
    fn subnet_membership_follows_the_prefix() {
        let c = parse_cidr("10.0.1.10/24").unwrap();
        assert!(c.contains(parse_ip("10.0.1.200").unwrap()));
        assert!(!c.contains(parse_ip("10.0.2.7").unwrap()));
        let wide = parse_cidr("10.0.1.10/16").unwrap();
        assert!(wide.contains(parse_ip("10.0.2.7").unwrap()));
        assert!(parse_cidr("0.0.0.0/0")
            .unwrap()
            .contains(parse_ip("192.168.4.4").unwrap()));
        assert_eq!(mask(24), 0xffffff00);
        assert_eq!(mask(32), u32::MAX);
    }

    #[test]
    fn macs_print_as_six_pairs() {
        assert_eq!(show_mac(0x0a), "02:00:00:00:00:0a");
        assert_eq!(show_mac(BROADCAST_MAC), "ff:ff:ff:ff:ff:ff");
    }
}
