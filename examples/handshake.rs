// #![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate std;
extern crate tokio;
use std::{
    net::SocketAddr,
    println,
    str
};
use core::net::{
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
};

use alloc::{
    vec::Vec,
    boxed::Box,
};
use futures::future::select_all;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        lookup_host, 
        TcpStream
    },
    // time::timeout,
};
use p2p_handshake::{
    protocol_builder::PayloadBuilder,
    errors::{
        self,
        ErrorSide::*
    },
    message::{
        payload::{
            VersionPayload,
        },
        header::MessageHeader,
    },
    COMMAND_SIZE, EMPTY_VERSION_SIZE, CUSTOM_VERSION_SIZE,
    traits::{
        EndianWrite,
        Builder,
    },
    helpers::le_checksum,
};

#[tokio::main]
async fn main() -> Result<(), errors::ErrorSide> {
    let resolved_addrs: Vec<_> = lookup_host(("seed.bitcoin.sipa.be", 8333)).await?.collect();
    let mut streams: Vec<_> = resolved_addrs
        .into_iter()
        .map(|x| version_handshake(x))
        .map(Box::pin)
        .collect();

    while !streams.is_empty() {
        match select_all(streams).await {
            (Ok(payload), _index, remaining) => {
                #[cfg(debug_assertions)]
                println!("Received Payload Length: Index {:?} Size {:?}", _index, payload.len());
                if payload.len() > 0 {
                    return Err(PayloadSizeMismatch(payload.len()));
                }
                streams = remaining;
            },
            (Err(e), _index, remaining) => {
                #[cfg(debug_assertions)]
                println!("Error : {}", e);
                streams = remaining;
            },
        };
    }

    Ok(())
}

async fn version_handshake(target: SocketAddr) -> Result<Vec<u8>, Box<dyn errors::Error>> {
    println!("Resolving for {:?}", target);
    let mut stream = TcpStream::connect(target).await?;
    println!("From {:?}", stream.local_addr()?.ip());
    let target = match target {
        SocketAddr::V4(v4_address) => {
            v4_address.ip().to_ipv6_mapped().octets()
        },
        SocketAddr::V6(v6_address) => {
            v6_address.ip().octets()
        }
    };
    let payload = PayloadBuilder::<VersionPayload>::init()
        .with_addr_recv(&target)?
        .with_addr_from(&Ipv4Addr::new(0,0,0,0).to_ipv6_mapped().octets())?
        .with_addr_from_port(0)?
        .build();
    #[cfg(debug_assertions)]
    println!("Default Payload {:?}", payload);
    let version_header = MessageHeader::version(payload.to_be_bytes())?.to_be_bytes_with_payload(&payload.to_be_bytes())?;
    let mut version_header_with_payload = [0_u8; 122]; // 24 + 98
    version_header_with_payload[..COMMAND_SIZE].copy_from_slice(&version_header);
    assert_eq!(payload.to_be_bytes().len(), CUSTOM_VERSION_SIZE);
    version_header_with_payload[COMMAND_SIZE..].copy_from_slice(&payload.to_be_bytes());
    //#[cfg(debug_assertions)]
    println!("Bytes to send {:?}", version_header_with_payload);
    println!("Bytes to send size {:?}", version_header_with_payload.len());
    let future_return = stream.write_all(&version_header_with_payload).await?;
    // read data from stream
    let buf_reader = BufReader::new(stream);
    let checked = check_bufread(buf_reader).await?;
    Ok(checked)
}

async fn check_bufread(mut payload: BufReader<TcpStream>) -> Result<Vec<u8>, Box<dyn errors::Error>> {
    let mut start_string = [0u8; 4];
    let mut command_name = [0u8; 12];
    let mut payload_size = [0u8; 4];
    let mut checksum = [0u8; 4];
    payload.read_exact(&mut start_string).await?;
    payload.read_exact(&mut command_name).await?;
    payload.read_exact(&mut payload_size).await?;
    payload.read_exact(&mut checksum).await?;
    
    let size = u32::from_be_bytes(payload_size);

    #[cfg(debug_assertions)]
    println!("Check size {:?}", size);
    let mut buf = Vec::with_capacity(size as usize);
    payload.read_to_end(&mut buf).await?;
    
    let message_header = MessageHeader {
        start_string,
        command_name,
        payload_size,
        checksum,
    };
    
    #[cfg(debug_assertions)]
    println!("Received Header {:?}", message_header);
    #[cfg(debug_assertions)]
    println!("With payload {:?}", buf);
    #[cfg(debug_assertions)]
    println!("With checksum {:?}", le_checksum(&buf));

    //let checksum = checksum.to_vec();
    //let mut payload_vec: Vec<u8> = Vec::new();
    let my_bytes = payload.fill_buf().await?;
    //payload.consume(24);
    // Read payload bytes
    //for _ in 0..payload_size {
    //    payload_vec.push(payload.read_u8().await?);
    //}
    //println!("RX Payload bytes {:?}", my_bytes.clone());
    Ok(my_bytes.to_vec())
}