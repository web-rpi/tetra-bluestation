#![allow(dead_code)]
mod config;
mod common;
mod entities;
mod saps;

mod testing;

use clap::Parser;

use crate::{common::{bitbuffer::BitBuffer, tdma_time::TdmaTime, tetra_common::Sap, tetra_entities::TetraEntity}, saps::{sapmsg::{SapMsg, SapMsgInner}, tmv::{TmvUnitdataInd, enums::logical_chans::LogicalChannel}}};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "TETRA Raw PDU Decoder",
    long_about = "Decodes a raw bitstring as a PDU for the specified SAP and destination component"
)]
struct Args {
    /// SAP (Service Access Point) name
    #[arg(
        help = "SAP name: [ Tmv ]"
    )]
    sap: String,

    /// Destination component name
    #[arg(
        help = "Destination component: [ Umac ]"
    )]
    destination: String,

    /// Raw bitstring to decode
    #[arg(
        help = "Raw bitstring (binary representation) to parse as PDU"
    )]
    bitstring: String,
}

fn main() {
    eprintln!("[+] Decoding tool");
    eprintln!("    Wouter Bokslag / Midnight Blue");

    let args = Args::parse();
    
    let _msg = match (args.sap.as_str(), args.destination.as_str()) {
        ("tmv", "umac") => {
            let pdu = TmvUnitdataInd {
                pdu: BitBuffer::from_bitstr(args.bitstring.as_str()),
                logical_channel: LogicalChannel::SchF,
                crc_pass: true,
                scrambling_code: 0,
            };
            SapMsg{ 
                sap: Sap::TmvSap,
                src: TetraEntity::Lmac,
                dest: TetraEntity::Umac,
                dltime: TdmaTime::default(),
                msg: SapMsgInner::TmvUnitdataInd(pdu)}
        },
        _ => {
            eprintln!("Error: Unsupported SAP '{}' or destination '{}'", args.sap, args.destination);
            std::process::exit(1);
        }
    };

    println!("Ready for {}-RESOURCE.ind PDU parsing implementation directed at {}...", args.sap.to_uppercase(), args.destination.to_uppercase());
}