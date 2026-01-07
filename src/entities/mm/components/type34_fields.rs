// use crate::common::bitbuffer::BitBuffer;
// use crate::common::pdu_parse_error::PduParseErr;
// use crate::entities::mm::enums::type34_elem_id_dl::MmType34ElemIdDl;
// use crate::entities::mm::enums::type34_elem_id_ul::MmType34ElemIdUl;
// use crate::common::typed::{parse_type3_generic};


// #[derive(Debug, PartialEq, Eq)]
// pub struct Type3FieldGeneric {
//     pub field_type: MmType34ElemIdDl,
//     pub len:   usize,
//     pub data:  u64,
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct Type3FieldGeneric {
//     pub field_type: MmType34ElemIdUl,
//     pub len:   usize,
//     pub data:  u64,
// }


// #[derive(Debug, PartialEq, Eq)]
// pub struct Type4FieldTodo {
//     pub field_type: MmType34ElemIdDl,
//     pub len:   usize,
//     pub num_elems: usize,
//     pub data:  u64,
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct Type4FieldTodo {
//     pub field_type: MmType34ElemIdUl,
//     pub len:   usize,
//     pub num_elems: usize,
//     pub data:  u64,
// }

// impl Type3FieldGeneric {
//     pub fn parse(obit: bool, buffer: &mut BitBuffer, expected_id: MmType34ElemIdDl) -> Result<Option<Self>, PduParseErr> { 
//         let id = expected_id.into_raw();
//         match parse_type3_generic(obit, buffer, id)? {
//             Some((len, data)) => Ok(Some(MmType3FieldDl { field_type: expected_id, len, data })),
//             None => Ok(None)
//         }
//     }

//     pub fn write(_buffer: &mut BitBuffer, _field_type: MmType34ElemIdDl, _data: u64, _len_bits: usize) {
//         unimplemented!();
//     }
// }

// impl MmType3FieldUl {
//     pub fn parse(buffer: &mut BitBuffer, expected_id: MmType34ElemIdUl) -> Result<Self, Type34Err> { 
//         let (len, data) = typed::parse_type3_generic(obit, buffer, expected_id.into_raw())?;
//         Ok(MmType3FieldUl { field_type: expected_id, len, data })
//     }

//     pub fn write(_buffer: &mut BitBuffer, _field_type: MmType34ElemIdUl, _data: u64, _len_bits: usize) {
//         unimplemented!();
//     }
// }

// impl MmType4FieldDl {

//     pub fn parse_header(buffer: &mut BitBuffer, expected_id: MmType34ElemIdDl) -> Result<(usize, usize), Type34Err> { 
//         parse_type4_header_generic(buffer, expected_id.into_raw())
//     }

//     pub fn parse(buffer: &mut BitBuffer, expected_id: MmType34ElemIdDl) -> Result<Self, Type34Err> { 
//         // Checks for mbit and id without moving buffer pos. If correct, reads len and num_elems, that we can then parse. 
//         let (len_bits, num_elems) = Self::parse_header(buffer, expected_id)?;

//         assert!(len_bits - 6 <= 64, "Type4 read too long, need Type4 refactor to accommodate this");

//         // tracing::debug!("MmType4FieldUl: num_elems: {}", num_elems);
//         let data = match buffer.read_bits(len_bits-6) {
//             Some(x) => x,
//             None => return Err(Type34Err::OutOfBounds),
//         };
//         // tracing::debug!("MmType4FieldUl: data: {}", data);
        
//         Ok(MmType4FieldDl { field_type: expected_id, len: len_bits, num_elems, data })
//     }

//     pub fn write(_buffer: &mut BitBuffer, _field_type: MmType34ElemIdDl, _repeated_elements: u64, _len_bits: usize) {
//         unimplemented!();
//     }

//     pub fn write_field(buffer: &mut BitBuffer, field_type: MmType34ElemIdDl, elems: &dyn Any) {
        
//         write_type34_header_generic(buffer, field_type.into_raw());

//         // Reserve length(11) + num_elems(6)
//         let pos_len_field = buffer.get_raw_pos();
//         buffer.write_bits(0, 11 + 6);

//         // Write payload and compute count
//         let num_elems: u64 = match field_type {
//             MmType34ElemIdDl::GroupIdentityDownlink => {
//                 let vec = elems
//                     .downcast_ref::<Vec<GroupIdentityDownlink>>()
//                     .expect("Expected Vec<GroupIdentityDownlink>");
//                 let n = vec.len() as u64;
//                 for elem in vec {
//                     elem.to_bitbuf(buffer);
//                 }
//                 n
//             }
//             _ => unimplemented!("Writing type4 field for {:?}", field_type),
//         };

//         // Backfill length and num_elems
//         let pos_end = buffer.get_raw_pos();
//         let len_bits = pos_end - pos_len_field - 11;

//         buffer.set_raw_pos(pos_len_field);
//         buffer.write_bits(len_bits as u64, 11);
//         buffer.write_bits(num_elems, 6);
//         buffer.set_raw_pos(pos_end);
//     }
// }

// impl MmType4FieldUl {
    
//     /// Checks for mbit and expected id without moving buffer pos. Returns error if not found.
//     /// If found, returns tuple with number of elements and total element length in bits.
//     /// The buffer position is advanced to the start of the first element. 
//     pub fn parse_header(buffer: &mut BitBuffer, expected_id: MmType34ElemIdUl) -> Result<(usize, usize), Type34Err> { 
//         parse_type4_header_generic(buffer, expected_id.into_raw())
//     }

//     pub fn parse(buffer: &mut BitBuffer, expected_id: MmType34ElemIdUl) -> Result<Self, Type34Err> { 

//         // Checks for mbit and id without moving buffer pos. If correct, reads len and num_elems, that we can then parse. 
//         let (len_bits, num_elems) = Self::parse_header(buffer, expected_id)?;

//         assert!(len_bits - 6 <= 64, "Type4 read too long, need Type4 refactor to accommodate this");

//         // tracing::debug!("MmType4FieldUl: num_elems: {}", num_elems);
//         let data = match buffer.read_bits(len_bits-6) {
//             Some(x) => x,
//             None => return Err(Type34Err::OutOfBounds),
//         };
//         // tracing::debug!("MmType4FieldUl: data: {}", data);
        
//         Ok(MmType4FieldUl { field_type: expected_id, len: len_bits, num_elems, data })
//     }

//     pub fn write(_buffer: &mut BitBuffer, _field_type: MmType34ElemIdUl, _repeated_elements: u64, _len_bits: usize) {
//         unimplemented!();
//     }


//     pub fn write_field(buffer: &mut BitBuffer, field_type: MmType34ElemIdUl, elems: &dyn Any) {
    
//         write_type34_header_generic(buffer, field_type.into_raw());

//         // Reserve length(11) + num_elems(6)
//         let pos_len_field = buffer.get_raw_pos();
//         buffer.write_bits(0, 11 + 6);

//         // Write payload and compute count
//         let num_elems: u64 = match field_type {
//             MmType34ElemIdUl::GroupIdentityUplink => {
//                 let vec = elems
//                     .downcast_ref::<Vec<GroupIdentityUplink>>()
//                     .expect("Expected Vec<GroupIdentityUplink>");
//                 let n = vec.len() as u64;
//                 for elem in vec {
//                     elem.to_bitbuf(buffer).expect("to_bitbuf failed");
//                 }
//                 n
//             }
//             _ => unimplemented!("Writing type4 field for {:?}", field_type),
//         };

//         // Backfill length and num_elems
//         let pos_end = buffer.get_raw_pos();
//         let len_bits = pos_end - pos_len_field - 11;

//         buffer.set_raw_pos(pos_len_field);
//         buffer.write_bits(len_bits as u64, 11);
//         buffer.write_bits(num_elems, 6);
//         buffer.set_raw_pos(pos_end);
//     }
// }
