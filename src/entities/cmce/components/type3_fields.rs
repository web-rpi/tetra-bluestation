// use crate::{common::{bitbuffer::BitBuffer, pdu_parse_error::PduParseErr, typed_pdu_fields::delimiters}, entities::cmce::enums::type3_elem_id::CmceType3ElemId};

// #[derive(Debug, PartialEq, Eq)]
// pub struct Type3FieldGeneric {
//     pub field_type: CmceType3ElemId,
//     pub len:   usize,
//     pub data:  u64,
// }

// impl Type3FieldGeneric {
    
//     /// Parse one Type-3 element.
//     /// Reads an M-bit. If 0, returns `Ok(None)`. If 1, reads 4-bit id, 11-bit length, then that many bits of data.
//     pub fn parse(buffer: &mut BitBuffer, field_name: &'static str) -> Result<Option<Self>, PduParseErr> {
//         let m = delimiters::read_mbit(buffer)?;
//         if m {
//             let id = buffer.read_field(4, field_name)?;
//             let id = CmceType3ElemId::try_from(id).map_err(|_| PduParseErr::InvalidElemId { found: id })?;
//             let len_bits = buffer.read_field(11, field_name)? as usize;
//             let data = buffer.read_field(len_bits, field_name)?;
//             Ok(Some(Type3FieldGeneric { field_type: id, len: len_bits, data }))
//         } else {
//             Ok(None)
//         }
//     }

//     /// Write one Type-3 element (as non-terminating).
//     /// Always writes M-bit=1, then 4-bit `id`, 11-bit `len`, then `len` bits from `data`.
//     pub fn write(_buffer: &mut BitBuffer, _field_type: CmceType3ElemId, _data: u64, _len_bits: usize) {
//         // self::write_mbit(buffer, 1);
//         // buffer.write_bits(field_type as u64, 4);
//         // buffer.write_bits(len_bits as u64, 11);
//         // buffer.write_bits(data, len_bits);
//         unimplemented!();
//     }
// }