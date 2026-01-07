use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseErr;
use crate::common::typed_pdu_fields::*;
use crate::entities::cmce::enums::type3_elem_id::CmceType3ElemId;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_dl::CmcePduTypeDl;

/// Representation of the D-CONNECT PDU (Clause 14.7.1.4).
/// This PDU shall be the order to the calling MS to through-connect.
/// Response expected: None
/// Response to: U-SETUP

// note 1: Basic service information element: If different from requested.
#[derive(Debug)]
pub struct DConnect {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 4 bits, Call time-out
    pub call_time_out: u8,
    /// Type1, 1 bits, Hook method selection
    pub hook_method_selection: bool,
    /// Type1, 1 bits, Simplex/duplex selection
    pub simplex_duplex_selection: bool,
    /// Type1, 2 bits, Transmission grant
    pub transmission_grant: u8,
    /// Type1, 1 bits, Transmission request permission
    pub transmission_request_permission: bool,
    /// Type1, 1 bits, Call ownership
    pub call_ownership: bool,
    /// Type2, 4 bits, Call priority
    pub call_priority: Option<u64>,
    /// Type2, 8 bits, See note,
    pub basic_service_information: Option<u64>,
    /// Type2, 24 bits, Temporary address
    pub temporary_address: Option<u64>,
    /// Type2, 6 bits, Notification indicator
    pub notification_indicator: Option<u64>,
    /// Type3, Facility
    pub facility: Option<Type3FieldGeneric>,
    /// Type3, Proprietary
    pub proprietary: Option<Type3FieldGeneric>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DConnect {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseErr> {
        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeDl::DConnect)?;
        
        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let call_time_out = buffer.read_field(4, "call_time_out")? as u8;
        // Type1
        let hook_method_selection = buffer.read_field(1, "hook_method_selection")? != 0;
        // Type1
        let simplex_duplex_selection = buffer.read_field(1, "simplex_duplex_selection")? != 0;
        // Type1
        let transmission_grant = buffer.read_field(2, "transmission_grant")? as u8;
        // Type1
        let transmission_request_permission = buffer.read_field(1, "transmission_request_permission")? != 0;
        // Type1
        let call_ownership = buffer.read_field(1, "call_ownership")? != 0;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = delimiters::read_obit(buffer)?;

        // Type2
        let call_priority = typed::parse_type2_generic(obit, buffer, 4, "call_priority")?;
        // Type2
        let basic_service_information = typed::parse_type2_generic(obit, buffer, 8, "basic_service_information")?;
        // Type2
        let temporary_address = typed::parse_type2_generic(obit, buffer, 24, "temporary_address")?;
        // Type2
        let notification_indicator = typed::parse_type2_generic(obit, buffer, 6, "notification_indicator")?;


        // Type3
        let facility = typed::parse_type3_generic(obit, buffer, CmceType3ElemId::Facility)?;

        // Type3
        let proprietary = typed::parse_type3_generic(obit, buffer, CmceType3ElemId::Proprietary)?;
        
        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseErr::InvalidTrailingMbitValue);
        }

        Ok(DConnect { 
            call_identifier, 
            call_time_out, 
            hook_method_selection, 
            simplex_duplex_selection, 
            transmission_grant, 
            transmission_request_permission, 
            call_ownership, 
            call_priority, 
            basic_service_information, 
            temporary_address, 
            notification_indicator, 
            facility, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseErr> {
        // PDU Type
        buffer.write_bits(CmcePduTypeDl::DConnect.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.call_time_out as u64, 4);
        // Type1
        buffer.write_bits(self.hook_method_selection as u64, 1);
        // Type1
        buffer.write_bits(self.simplex_duplex_selection as u64, 1);
        // Type1
        buffer.write_bits(self.transmission_grant as u64, 2);
        // Type1
        buffer.write_bits(self.transmission_request_permission as u64, 1);
        // Type1
        buffer.write_bits(self.call_ownership as u64, 1);

        // Check if any optional field present and place o-bit
        let obit = self.call_priority.is_some() || self.basic_service_information.is_some() || self.temporary_address.is_some() || self.notification_indicator.is_some() || self.facility.is_some() || self.proprietary.is_some() ;
        delimiters::write_obit(buffer, obit as u8);
        if !obit { return Ok(()); }

        // Type2
        typed::write_type2_generic(obit, buffer, self.call_priority, 4);

        // Type2
        typed::write_type2_generic(obit, buffer, self.basic_service_information, 8);

        // Type2
        typed::write_type2_generic(obit, buffer, self.temporary_address, 24);

        // Type2
        typed::write_type2_generic(obit, buffer, self.notification_indicator, 6);

        // Type3
        typed::write_type3_generic(obit, buffer, &self.facility, CmceType3ElemId::Facility)?;

        // Type3
        typed::write_type3_generic(obit, buffer, &self.proprietary, CmceType3ElemId::Proprietary)?;
        
        // Write terminating m-bit
        delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for DConnect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DConnect {{ call_identifier: {:?} call_time_out: {:?} hook_method_selection: {:?} simplex_duplex_selection: {:?} transmission_grant: {:?} transmission_request_permission: {:?} call_ownership: {:?} call_priority: {:?} basic_service_information: {:?} temporary_address: {:?} notification_indicator: {:?} facility: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.call_time_out,
            self.hook_method_selection,
            self.simplex_duplex_selection,
            self.transmission_grant,
            self.transmission_request_permission,
            self.call_ownership,
            self.call_priority,
            self.basic_service_information,
            self.temporary_address,
            self.notification_indicator,
            self.facility,
            self.proprietary,
        )
    }
}
