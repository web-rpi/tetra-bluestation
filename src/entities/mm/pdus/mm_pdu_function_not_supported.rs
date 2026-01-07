use core::fmt;

use crate::common::pdu_parse_error::PduParseErr;
use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields::*;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_pdu_type_dl::MmPduTypeDl;

/// Representation of the MM PDU/FUNCTION NOT SUPPORTED PDU (Clause 16.9.4.1).
/// This PDU may be sent by the MS/LS or SwMI to indicate that the received MM PDU or the function indicated in the PDU is not supported.
/// Response expected: -
/// Response to: Any individually addressed MM PDU

// note 1: This information element shall identify the received PDU which contains the function which cannot be supported.
// note 2: In case the receiving party recognizes the PDU and the PDU contains a sub-PDU field (like in U/M-MM STATUS PDU, U/D-OTAR, U/D-ENABLE, etc.) this element contains the element indicating which sub-PDU this is.
// note 3: The length of this element is indicated by the Length of the copied PDU element. This element is not present if the Length of the copied PDU element is not present.
// note 4: This element contains the received PDU beginning from and excluding the PDU type element.
#[derive(Debug)]
pub struct MmPduFunctionNotSupported {
    /// Type1, 4 bits, See note 1,
    pub not_supported_pdu_type: u8,
    /// Type2, See note 2,
    pub not_supported_sub_pdu_type: Option<u64>,
    /// Type2, 8 bits, Length of the copied PDU
    pub length_of_the_copied_pdu: Option<u64>,
    /// Conditional See notes 3 and 4,
    pub received_pdu_contents: Option<u64>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
#[allow(unused_variables)]
impl MmPduFunctionNotSupported {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseErr> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeDl::MmPduFunctionNotSupported)?;
        
        // Type1
        let not_supported_pdu_type = buffer.read_field(4, "not_supported_pdu_type")? as u8;

        // obit designates presence of any further type2, type3 or type4 fields
        let obit = delimiters::read_obit(buffer)?;

        // Type2
        unimplemented!();
        let not_supported_sub_pdu_type = typed::parse_type2_generic(obit, buffer, 999, "not_supported_sub_pdu_type")?;
        // Type2
        let length_of_the_copied_pdu = typed::parse_type2_generic(obit, buffer, 8, "length_of_the_copied_pdu")?;
        // Conditional
        unimplemented!(); let received_pdu_contents = if obit { Some(0) } else { None };

        // Read trailing obit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseErr::InvalidTrailingMbitValue);
        }

        Ok(MmPduFunctionNotSupported { 
            not_supported_pdu_type, 
            not_supported_sub_pdu_type, 
            length_of_the_copied_pdu, 
            received_pdu_contents
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseErr> {
        // PDU Type
        buffer.write_bits(MmPduTypeDl::MmPduFunctionNotSupported.into_raw(), 4);
        // Type1
        buffer.write_bits(self.not_supported_pdu_type as u64, 4);

        // Check if any optional field present and place o-bit
        let obit = self.not_supported_sub_pdu_type.is_some() || self.length_of_the_copied_pdu.is_some() ;
        delimiters::write_obit(buffer, obit as u8);
        if !obit { return Ok(()); }

        // Type2
        unimplemented!();
            typed::write_type2_generic(obit, buffer, self.not_supported_sub_pdu_type, 999);

        // Type2
        typed::write_type2_generic(obit, buffer, self.length_of_the_copied_pdu, 8);

        // Conditional
        if let Some(ref _value) = self.received_pdu_contents {
            unimplemented!();
            buffer.write_bits(*_value, 999);
        }
        // Write terminating m-bit
        delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for MmPduFunctionNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MmPduFunctionNotSupported {{ not_supported_pdu_type: {:?} not_supported_sub_pdu_type: {:?} length_of_the_copied_pdu: {:?} received_pdu_contents: {:?} }}",
            self.not_supported_pdu_type,
            self.not_supported_sub_pdu_type,
            self.length_of_the_copied_pdu,
            self.received_pdu_contents,
        )
    }
}
