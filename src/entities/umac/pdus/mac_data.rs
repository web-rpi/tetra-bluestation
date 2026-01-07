use core::fmt;

use crate::{common::{address::{SsiType, TetraAddress}, bitbuffer::BitBuffer}, entities::umac::enums::reservation_requirement::ReservationRequirement};
use crate::common::pdu_parse_error::PduParseErr;

/// Clause 21.4.2.3 MAC-DATA
#[derive(Debug, Clone)]
pub struct MacData {
    // 1
    pub fill_bits: bool,
    // 1
    pub encrypted: bool,
    // 2
    // pub addr_type: u8,
    // 24 opt
    // pub ssi: Option<u32>,
    // 10 opt
    pub event_label: Option<u16>,
    pub addr: TetraAddress,
    /// 6 bit, optional. If not provided, frag_flag and reservation_req must be provided
    pub length_ind: Option<u8>,
    /// 1 bit, optional. If not provided, length_ind must be provided
    pub frag_flag: Option<bool>,
    /// 4 opt, optional. If not provided, length_ind must be provided
    pub reservation_req: Option<ReservationRequirement>,
}

impl MacData {
    pub fn from_bitbuf(buf: &mut BitBuffer) -> Result<Self, PduParseErr> {
        let mut s = MacData {
            fill_bits: false,
            encrypted: false,
            // addr_type: 0,
            // ssi: None,
            event_label: None,
            addr: TetraAddress::default(),
            length_ind: None,
            frag_flag: None,
            reservation_req: None,
            // RESERVED: None,
        };

        // required constant mac_pdu_type
        assert!(buf.read_field(2, "mac_pdu_type")? == 0);
        s.fill_bits = buf.read_field(1, "fill_bits")? != 0;
        s.encrypted = buf.read_field(1, "encrypted")? != 0;
        s.addr.encrypted = s.encrypted;
        
        let addr_type = buf.read_field(2, "addr_type")? as u8;
        match addr_type {
            0 => {
                s.addr.ssi_type = SsiType::Ssi;
                s.addr.ssi = buf.read_field(24, "ssi")? as u32;
            }
            1 => {
                s.event_label = Some(buf.read_field(10, "event_label")? as u16);
            }
            2 => {
                s.addr.ssi_type = SsiType::Ussi;
                s.addr.ssi = buf.read_field(24, "ussi")? as u32;
            }
            3 => {
                s.addr.ssi_type = SsiType::Smi;
                s.addr.ssi = buf.read_field(24, "smi")? as u32;
            }
            _ => {
                panic!();
            }
        }

        let length_ind_or_cap_req = buf.read_field(1, "length_ind_or_cap_req")?;
        if length_ind_or_cap_req == 0 { 
            s.length_ind = Some(buf.read_field(6, "length_ind")? as u8); 
        } else {
            s.frag_flag = Some(buf.read_field(1, "frag_flag")? != 0); 
            let val = buf.read_field(4, "reservation_requirement")?;
            s.reservation_req = Some(ReservationRequirement::try_from(val).expect("invalid reservation request"));
            buf.read_bits(1); // Reserved bit
        }

        Ok(s)
    }

    pub fn to_bitbuf(&self, buf: &mut BitBuffer) {

        // write required constant mac_pdu_type
        buf.write_bits(0, 2);
        buf.write_bits(self.fill_bits as u8 as u64, 1);
        buf.write_bits(self.encrypted as u8 as u64, 1);
        assert!(self.addr.encrypted == self.encrypted);
        
        // Derive addr_type from addr and write type and field
        match self.addr.ssi_type {
            SsiType::Ssi => {
                buf.write_bits(0, 2);
                buf.write_bits(self.addr.ssi as u64, 24);
            }
            SsiType::Ussi => {
                buf.write_bits(2, 2);
                buf.write_bits(self.addr.ssi as u64, 24);
            }
            SsiType::Smi => {
                buf.write_bits(3, 2);
                buf.write_bits(self.addr.ssi as u64, 24);
            }
            _ => {
                // We must have an event label
                buf.write_bits(1, 2);
                buf.write_bits(self.event_label.unwrap() as u64, 10);
            }
        };
        
        if self.length_ind.is_some() {
            buf.write_bits(0, 1); // length_ind_or_cap_req
            buf.write_bits(self.length_ind.unwrap() as u64, 6);
        } else {
            assert!(self.frag_flag.is_some() && self.reservation_req.is_some());
            buf.write_bits(1, 1); // length_ind_or_cap_req
            buf.write_bits(self.frag_flag.unwrap() as u64, 1);
            buf.write_bits(self.reservation_req.unwrap() as u64, 4);
            buf.write_bits(0, 1); // Reserved bit
        }
    }
}

impl fmt::Display for MacData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MacData {{ fill_bits: {} encrypted: {} addr: {}", self.fill_bits, self.encrypted, self.addr)?;
        if let Some(v) = self.event_label { 
            write!(f, " event_label: {}", v)?; 
        }
        if let Some(v) = self.length_ind { 
            write!(f, " length_ind: {}", v)?; 
        }
        if let Some(v) = self.frag_flag { 
            write!(f, " frag_flag: {}", v)?; 
        }
        if let Some(v) = self.reservation_req { 
            write!(f, " reservation_req: {}", v)?; 
        }
        write!(f, " }}")
    }
}
