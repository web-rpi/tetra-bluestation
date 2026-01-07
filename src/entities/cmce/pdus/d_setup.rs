use core::fmt;

use crate::common::bitbuffer::BitBuffer;
use crate::common::pdu_parse_error::PduParseErr;
use crate::common::typed_pdu_fields::*;
use crate::entities::cmce::enums::type3_elem_id::CmceType3ElemId;
use crate::expect_pdu_type;
use crate::entities::cmce::enums::cmce_pdu_type_dl::CmcePduTypeDl;

/// Representation of the D-SETUP PDU (Clause 14.7.1.12).
/// This PDU shall be the call set-up message sent to the called MS.
/// Response expected: U-ALERT/U-CONNECT/-
/// Response to: -

// note 1: This information element is used by SS-PC, refer to ETSI EN 300 392-12-10 [15] and SS-PPC and ETSI EN 300 392-12-16 [16].
// note 2: For resolution of possible Facility (Talking Party Identifier)/Calling party identifier conflicts, refer to ETSI EN 300 392-12-3 [12], clause 5.2.1.5 and ETSI EN 300 392-12-1 [11], clause 4.3.5.
// note 3: Shall be conditional on the value of Calling Party Type Identifier (CPTI): • CPTI = 1 ⇒ Calling Party SSI; • CPTI = 2 ⇒ Calling Party SSI + Calling Party Extension.
#[derive(Debug)]
pub struct DSetup {
    /// Type1, 14 bits, Call identifier
    pub call_identifier: u16,
    /// Type1, 4 bits, Call time-out
    pub call_time_out: u8,
    /// Type1, 1 bits, Hook method selection
    pub hook_method_selection: bool,
    /// Type1, 1 bits, Simplex/duplex selection
    pub simplex_duplex_selection: bool,
    /// Type1, 8 bits, Basic service information
    pub basic_service_information: u8,
    /// Type1, 2 bits, Transmission grant
    pub transmission_grant: u8,
    /// Type1, 1 bits, Transmission request permission
    pub transmission_request_permission: bool,
    /// Type1, 4 bits, See note 1,
    pub call_priority: u8,
    /// Type2, 6 bits, Notification indicator
    pub notification_indicator: Option<u64>,
    /// Type2, 24 bits, Temporary address
    pub temporary_address: Option<u64>,
    /// Type2, 2 bits, See note 2,
    pub calling_party_type_identifier: Option<u64>,
    /// Conditional 24 bits, See note 3, condition: calling_party_type_identifier == Some(1) || calling_party_type_identifier == Some(2)
    pub calling_party_address_ssi: Option<u64>,
    /// Conditional 24 bits, See note 3, condition: calling_party_type_identifier == Some(2)
    pub calling_party_extension: Option<u64>,
    /// Type3, External subscriber number
    pub external_subscriber_number: Option<Type3FieldGeneric>,
    /// Type3, Facility
    pub facility: Option<Type3FieldGeneric>,
    /// Type3, DM-MS address
    pub dm_ms_address: Option<Type3FieldGeneric>,
    /// Type3, Proprietary
    pub proprietary: Option<Type3FieldGeneric>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
impl DSetup {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseErr> {

        let pdu_type = buffer.read_field(5, "pdu_type")?;
        expect_pdu_type!(pdu_type, CmcePduTypeDl::DSetup)?;

        // Type1
        let call_identifier = buffer.read_field(14, "call_identifier")? as u16;
        // Type1
        let call_time_out = buffer.read_field(4, "call_time_out")? as u8;
        // Type1
        let hook_method_selection = buffer.read_field(1, "hook_method_selection")? != 0;
        // Type1
        let simplex_duplex_selection = buffer.read_field(1, "simplex_duplex_selection")? != 0;
        // Type1
        let basic_service_information = buffer.read_field(8, "basic_service_information")? as u8;
        // Type1
        let transmission_grant = buffer.read_field(2, "transmission_grant")? as u8;
        // Type1
        let transmission_request_permission = buffer.read_field(1, "transmission_request_permission")? != 0;
        // Type1
        let call_priority = buffer.read_field(4, "call_priority")? as u8;

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = delimiters::read_obit(buffer)?;

        // Type2
        let notification_indicator = typed::parse_type2_generic(obit, buffer, 6, "notification_indicator")?;
        // Type2
        let temporary_address = typed::parse_type2_generic(obit, buffer, 24, "temporary_address")?;
        // Type2
        let calling_party_type_identifier = typed::parse_type2_generic(obit, buffer, 2, "calling_party_type_identifier")?;
        // Conditional
        let calling_party_address_ssi = if obit && calling_party_type_identifier == Some(1) || calling_party_type_identifier == Some(2) { 
            Some(buffer.read_field(24, "calling_party_address_ssi")?) 
        } else { None };
        // Conditional
        let calling_party_extension = if obit && calling_party_type_identifier == Some(2) { 
            Some(buffer.read_field(24, "calling_party_extension")?) 
        } else { None };


        // Type3
        let external_subscriber_number = typed::parse_type3_generic(obit, buffer, CmceType3ElemId::ExtSubscriberNum)?;
        
        // Type3
        let facility = typed::parse_type3_generic(obit, buffer, CmceType3ElemId::Facility)?;
        
        // Type3
        let dm_ms_address = typed::parse_type3_generic(obit, buffer, CmceType3ElemId::DmMsAddr)?;
        
        // Type3
        let proprietary = typed::parse_type3_generic(obit, buffer, CmceType3ElemId::Proprietary)?;
        
        
        // Read trailing mbit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseErr::InvalidTrailingMbitValue);
        }

        Ok(DSetup { 
            call_identifier, 
            call_time_out, 
            hook_method_selection, 
            simplex_duplex_selection, 
            basic_service_information, 
            transmission_grant, 
            transmission_request_permission, 
            call_priority, 
            notification_indicator, 
            temporary_address, 
            calling_party_type_identifier, 
            calling_party_address_ssi, 
            calling_party_extension, 
            external_subscriber_number, 
            facility, 
            dm_ms_address, 
            proprietary 
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseErr> {
        // PDU Type
        buffer.write_bits(CmcePduTypeDl::DSetup.into_raw(), 5);
        // Type1
        buffer.write_bits(self.call_identifier as u64, 14);
        // Type1
        buffer.write_bits(self.call_time_out as u64, 4);
        // Type1
        buffer.write_bits(self.hook_method_selection as u64, 1);
        // Type1
        buffer.write_bits(self.simplex_duplex_selection as u64, 1);
        // Type1
        buffer.write_bits(self.basic_service_information as u64, 8);
        // Type1
        buffer.write_bits(self.transmission_grant as u64, 2);
        // Type1
        buffer.write_bits(self.transmission_request_permission as u64, 1);
        // Type1
        buffer.write_bits(self.call_priority as u64, 4);

        // Check if any optional field present and place o-bit
        let obit = self.notification_indicator.is_some() || self.temporary_address.is_some() || self.calling_party_type_identifier.is_some() || self.external_subscriber_number.is_some() || self.facility.is_some() || self.dm_ms_address.is_some() || self.proprietary.is_some() ;
        delimiters::write_obit(buffer, obit as u8);
        if !obit { return Ok(()); }

        // Type2
        typed::write_type2_generic(obit, buffer, self.notification_indicator, 6);

        // Type2
        typed::write_type2_generic(obit, buffer, self.temporary_address, 24);

        // Type2
        typed::write_type2_generic(obit, buffer, self.calling_party_type_identifier, 2);

        // Conditional
        if let Some(ref value) = self.calling_party_address_ssi {
            buffer.write_bits(*value, 24);
        }
        // Conditional
        if let Some(ref value) = self.calling_party_extension {
            buffer.write_bits(*value, 24);
        }
        // Type3
        typed::write_type3_generic(obit, buffer, &self.external_subscriber_number, CmceType3ElemId::ExtSubscriberNum)?;
        
        // Type3
        typed::write_type3_generic(obit, buffer, &self.facility, CmceType3ElemId::Facility)?;
        // Type3
        typed::write_type3_generic(obit, buffer, &self.dm_ms_address, CmceType3ElemId::DmMsAddr)?;
        
        // Type3
        typed::write_type3_generic(obit, buffer, &self.proprietary, CmceType3ElemId::Proprietary)?;

        // Write terminating m-bit
        delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for DSetup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DSetup {{ call_identifier: {:?} call_time_out: {:?} hook_method_selection: {:?} simplex_duplex_selection: {:?} basic_service_information: {:?} transmission_grant: {:?} transmission_request_permission: {:?} call_priority: {:?} notification_indicator: {:?} temporary_address: {:?} calling_party_type_identifier: {:?} calling_party_address_ssi: {:?} calling_party_extension: {:?} external_subscriber_number: {:?} facility: {:?} dm_ms_address: {:?} proprietary: {:?} }}",
            self.call_identifier,
            self.call_time_out,
            self.hook_method_selection,
            self.simplex_duplex_selection,
            self.basic_service_information,
            self.transmission_grant,
            self.transmission_request_permission,
            self.call_priority,
            self.notification_indicator,
            self.temporary_address,
            self.calling_party_type_identifier,
            self.calling_party_address_ssi,
            self.calling_party_extension,
            self.external_subscriber_number,
            self.facility,
            self.dm_ms_address,
            self.proprietary,
        )
    }
}
