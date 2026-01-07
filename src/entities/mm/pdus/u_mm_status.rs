use core::fmt;

use crate::common::pdu_parse_error::PduParseErr;

use crate::common::bitbuffer::BitBuffer;
use crate::common::typed_pdu_fields::*;
use crate::expect_pdu_type;
use crate::entities::mm::enums::mm_pdu_type_ul::MmPduTypeUl;

/// Representation of the U-MM STATUS PDU (Clause 16.9.3.5.1).
/// The MS sends this message to the infrastructure to request or respond to a mode change.
/// Response expected: -/D-MM STATUS
/// Response to: -/D-MM STATUS

// note 1: This information element shall indicate the requested service or a response to a request and the sub-type of the U-MM STATUS PDU.
// note 2: This information element or set of information elements shall be as defined by the status uplink information element, refer to clauses 16.9.3.5.1 to 16.9.3.5.8.
// note 3: This Status uplink element indicates which sub-PDU this U-MM STATUS PDU contains; in case the receiving party does not support indicated function but recognizes this PDU structure, it should set the received value of Status uplink element to Not-supported sub PDU type element.
#[derive(Debug)]
pub struct UMmStatus {
    /// Type1, 6 bits, See notes 1 and 3,
    pub status_uplink: u8,
    /// Conditional See note 2,
    pub status_uplink_dependent_information: Option<u64>,
}

#[allow(unreachable_code)] // TODO FIXME review, finalize and remove this
#[allow(unused_variables)]
impl UMmStatus {
    /// Parse from BitBuffer
    pub fn from_bitbuf(buffer: &mut BitBuffer) -> Result<Self, PduParseErr> {

        let pdu_type = buffer.read_field(4, "pdu_type")?;
        expect_pdu_type!(pdu_type, MmPduTypeUl::UMmStatus)?;
        
        // Type1
        let status_uplink = buffer.read_field(6, "status_uplink")? as u8;
        // Conditional
        unimplemented!(); let status_uplink_dependent_information = if true { Some(0) } else { None };

        // obit designates presence of any further type2, type3 or type4 fields
        let mut obit = delimiters::read_obit(buffer)?;


        // Read trailing obit (if not previously encountered)
        obit = if obit { buffer.read_field(1, "trailing_obit")? == 1 } else { obit };
        if obit {
            return Err(PduParseErr::InvalidTrailingMbitValue);
        }

        Ok(UMmStatus { 
            status_uplink, 
            status_uplink_dependent_information
        })
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseErr> {
        // PDU Type
        buffer.write_bits(MmPduTypeUl::UMmStatus.into_raw(), 4);
        // Type1
        buffer.write_bits(self.status_uplink as u64, 6);
        // Conditional
        if let Some(ref value) = self.status_uplink_dependent_information {
            unimplemented!();
            buffer.write_bits(*value, 999);
        }
        // Write terminating m-bit
        delimiters::write_mbit(buffer, 0);
        Ok(())
    }
}

impl fmt::Display for UMmStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UMmStatus {{ status_uplink: {:?} status_uplink_dependent_information: {:?} }}",
            self.status_uplink,
            self.status_uplink_dependent_information,
        )
    }
}
