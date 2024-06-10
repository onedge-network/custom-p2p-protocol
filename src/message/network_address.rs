use crate::{
    errors::{
        ErrorSide,
        Error
    },
    traits::{
        EndianWrite,
        Length
    },
};
use core::net::{
    Ipv4Addr,
    Ipv6Addr,
};
use core::mem;

// Network Data Layout Size Constants for runtime.
pub const NETWORK_TIME: usize = 4;
pub const NETWORK_SERVICES: usize = 8;
pub const NETWORK_IPvXX: usize = 16;
pub const NETWORK_PORT: usize = 2;

// 
pub const DEFAULT_IPADDR: [u8; NETWORK_IPvXX] = Ipv4Addr::new(127, 0, 0, 1).to_ipv6_mapped().octets();
pub const DEFAULT_PORT: u16 = 8333;

// Defines variants for IPv4 and IPv6.
pub enum IP {
    V4([u8; NETWORK_IPvXX]),
    V6([u8; NETWORK_IPvXX]),
}

impl EndianWrite for IP {
    type Output = [u8; NETWORK_IPvXX];
    fn to_le_bytes(&self) -> Self::Output {
        match self {
            Self::V4(network_address) | Self::V6(network_address) => {
                let mut buf: Self::Output = *network_address;
                buf.reverse();
                buf
            },
        }
    }
    fn to_be_bytes(&self) -> Self::Output {
        match self {
            Self::V4(network_address) | Self::V6(network_address) => {
                *network_address
            },
        }
    }
}

#[derive(Clone)]
pub enum NetworkOptions {
    NetworkTime(Option<[u8;NETWORK_TIME]>),
    NetworkServices(Option<[u8;NETWORK_SERVICES]>),
    NetworkIpvXX(Option<[u8;NETWORK_IPvXX]>),
    NetworkPort(Option<[u8;NETWORK_PORT]>),
}

impl Length for NetworkOptions {
    fn len(&self) -> usize {
        match self {
            NetworkOptions::NetworkTime(option) => {
                match option {
                    None => 0_usize,
                    Some(serial_layout) => {
                        serial_layout.len()
                    }
                }
            },
            NetworkOptions::NetworkServices(option) => {
                match option {
                    None => 0_usize,
                    Some(serial_layout) => {
                        serial_layout.len()
                    }
                }
            },
            NetworkOptions::NetworkIpvXX(option) => {
                match option {
                    None => 0_usize,
                    Some(serial_layout) => {
                        serial_layout.len()
                    }
                }
            },
            NetworkOptions::NetworkPort(option) => {
                match option {
                    None => 0_usize,
                    Some(serial_layout) => {
                        serial_layout.len()
                    }
                }
            },
        }
    }
}

// NetworkTime(Option<[u8;NETWORK_TIME]>),
// NetworkServices(Option<[u8;NETWORK_SERVICES]>),
// NetworkIpvXX(Option<[u8;NETWORK_IPvXX]>),
// NetworkPort(Option<[u8;NETWORK_PORT]>),
impl EndianWrite for NetworkOptions {
    type Output = Vec<u8>;
    fn to_le_bytes(&self) -> Self::Output {
        let mut options = self.to_be_bytes();
        options.reverse();
        options
    }
    fn to_be_bytes(&self) -> Self::Output {
        let options: Vec<u8> = match self {
            NetworkOptions::NetworkTime(option) => {
                match option {
                    None => Vec::new(),
                    Some(serial_layout) => {
                        (*serial_layout).to_vec()
                    }
                }
            },
            NetworkOptions::NetworkServices(option) => {
                match option {
                    None => Vec::new(),
                    Some(serial_layout) => {
                        (*serial_layout).to_vec()
                    }
                }
            },
            NetworkOptions::NetworkIpvXX(option) => {
                match option {
                    None => Vec::new(),
                    Some(serial_layout) => {
                        (*serial_layout).to_vec()
                    }
                }
            },
            NetworkOptions::NetworkPort(option) => {
                match option {
                    None => Vec::new(),
                    Some(serial_layout) => {
                        (*serial_layout).to_vec()
                    }
                }
            },
        };
        #[cfg(debug_assertions)]
        println!("Network Options {:?}", options);
        options
    }
}

// Defines use of network address (they differ)
#[derive(Clone)]
pub enum NetworkAddress { 
    NonVersion(
        [NetworkOptions;4]
    ),
    Version(
        [NetworkOptions;4]
    ),
}

impl NetworkAddress {
    pub fn set_ip(&mut self, ip: &[u8; NETWORK_IPvXX]) -> Result<[u8;NETWORK_IPvXX], Box<dyn Error>> {
        match self {
            Self::Version(options) | Self::NonVersion(options) => {
                options[3] = NetworkOptions::NetworkIpvXX(Some(ip.clone()));
            },
            _ => {
                return Err(Box::new(ErrorSide::Unreachable))
            }
        }
        Ok(ip.clone())
    }
}

impl Length for NetworkAddress {
    fn len(&self) -> usize {
        match self {
            Self::NonVersion(options)
            | Self::Version(options) => {
                options
                    .into_iter()
                    .map(|x| {x.len()} )
                    .sum::<usize>()
            },
        }
    }
}

impl EndianWrite for NetworkAddress {
    type Output = Vec<u8>;
    fn to_le_bytes(&self) -> Self::Output {
        let mut options = self.to_be_bytes();
        options.reverse();
        options
    }
    fn to_be_bytes(&self) -> Self::Output {
        let mut options = match self {
            Self::NonVersion(options)
            | Self::Version(options)  => {
                options
                    .into_iter()
                    .map(|x| {x.to_be_bytes()} )  // TODO: check double endianess
                    .flatten()
                    .collect::<Self::Output>()
            },
        };
        options
    }
}

pub enum Services {
    NODE_NETWORK = 0x01,
    NODE_GETUTXO = 0x02,
    NODE_BLOOM = 0x04,
    NODE_WITNESS = 0x08,
    NODE_XTHIN = 0x10,
    NODE_COMPACT_FILTERS = 0x40,
    NODE_NETWORK_LIMITED = 0x0400,
}

impl EndianWrite for Services {
    type Output = [u8;NETWORK_SERVICES];
    fn to_le_bytes(&self) -> Self::Output {
        let mut buf: Self::Output = self.to_be_bytes();
        buf.reverse();
        buf
    }
    fn to_be_bytes(&self) -> Self::Output {
        let mut buf: Self::Output = Self::Output::default();
        let a: i64 = match self {
            Services::NODE_NETWORK => Services::NODE_NETWORK as i64,
            Services::NODE_GETUTXO => Services::NODE_GETUTXO as i64,
            Services::NODE_BLOOM => Services::NODE_BLOOM as i64,
            Services::NODE_WITNESS => Services::NODE_WITNESS as i64,
            Services::NODE_XTHIN => Services::NODE_XTHIN as i64,
            Services::NODE_COMPACT_FILTERS => Services::NODE_COMPACT_FILTERS as i64,
            Services::NODE_NETWORK_LIMITED => Services::NODE_NETWORK_LIMITED as i64,
        };
        assert_eq!(a.to_be_bytes().len(), 8);
        buf[0..8].clone_from_slice(&a.to_be_bytes());
        a.to_be_bytes()
    }
}

 // Default for all variants
impl Default for NetworkAddress {
    fn default() -> Self {
        NetworkAddress::Version(
            [
                NetworkOptions::NetworkTime(None), 
                NetworkOptions::NetworkServices(Some(Services::NODE_NETWORK.to_le_bytes())), 
                NetworkOptions::NetworkIpvXX(Some(DEFAULT_IPADDR)), 
                NetworkOptions::NetworkPort(Some(DEFAULT_PORT.to_be_bytes()))
            ]
        )
    }
}


// #[test]
// fn check_network_ip_sizes() {
//     todo!()
// }