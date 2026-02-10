use tetra_core::{BitBuffer, PhyBlockType, unimplemented_log};
use tetra_saps::tmv::TmvUnitdataReq;
use tetra_saps::tmv::enums::logical_chans::LogicalChannel;
use tetra_saps::tp::TpUnitdataInd;

use crate::lmac::components::convenc::{self, ConvEncState, RcpcPunctMode};
use crate::lmac::components::{crc16, errorcontrol_params, interleaver, rm3014, viterbi};
use crate::lmac::components::scrambler;

const MAX_TYPE1_BITS: usize = 268;
const MAX_TYPE2_BITS: usize = 288;

const MAX_TYPE345_BITS: usize = 432;
const MAX_TYPE345_HALFSLOT_BITS: usize = 216;


/// Encodes control plane message from type1 to type5 bits
/// Handles CP channels except AACH
pub fn encode_cp(mut prim: TmvUnitdataReq) -> BitBuffer {

    let lchan = prim.logical_channel;
    assert!(lchan.is_control_channel() && lchan != LogicalChannel::Aach);

    let params = errorcontrol_params::get_params(lchan);

    assert!(prim.mac_block.get_len() == params.type1_bits, 
        "encode_cp: prim.mac_block length {} does not match type1_bits {} for lchan {:?}",
        prim.mac_block.get_len(), params.type1_bits, lchan);
    tracing::trace!("encode_cp {:?} type1 {}", lchan, prim.mac_block.dump_bin());

    // Convert bitbuffer to bitarray -> type1
    prim.mac_block.seek(0);
    let mut type2_arr = [0u8; MAX_TYPE2_BITS]; // largest possible type1 block
    prim.mac_block.to_bitarr(&mut type2_arr[0..params.type1_bits]);

    // CRC addition, type1 -> type2
    assert!(params.have_crc16);
    let crc = !crc16::crc16_ccitt_bits( &type2_arr[0..params.type1_bits], params.type1_bits);
    for i in 0..16 {
        type2_arr[params.type1_bits + i] = ((crc >> (15 - i)) & 1) as u8;
    }
    tracing::trace!("encode_cp {:?} type2: {:?}", lchan, BitBuffer::from_bitarr(&type2_arr[0..params.type2_bits]).dump_bin());

    // Viterbi, type2 -> type3dp
    let mut type3dp_arr = [0u8; MAX_TYPE345_BITS*4];
    let mut ces = ConvEncState::new();
    ces.encode(&type2_arr[0..params.type2_bits], &mut type3dp_arr);
    // tracing::trace!("encode_cp: t3dp:  {:?}", &type3dp_arr[0..4*params.type2_bits]);

    // Puncturing, type3dp -> type3
    let mut type3_arr = [0u8; MAX_TYPE345_BITS]; // Need params.type345_bits
    convenc::get_punctured_rate(RcpcPunctMode::Rate2_3, &type3dp_arr, &mut type3_arr);
    tracing::trace!("encode_cp {:?} type3: {:?}", lchan, BitBuffer::from_bitarr(&type3_arr[0..params.type345_bits]).dump_bin());

    // Interleaving, type3 -> type4
    let mut type4_arr = [0u8; MAX_TYPE345_BITS]; // Need params.type345_bits
    interleaver::block_interleave(params.type345_bits, params.interleave_a, &type3_arr, &mut type4_arr);
    let mut type4 = BitBuffer::from_bitarr(&type4_arr[0..params.type345_bits]);
    tracing::trace!("encode_cp {:?} type4: {:?}", lchan, type4.dump_bin());

    // Scrambling, type4 -> type5
    scrambler::tetra_scramb_bits(prim.scrambling_code, &mut type4);
    let type5 = type4;
    tracing::trace!("encode_cp {:?} type5: {:?}", lchan, type5.dump_bin());

    // Pass block to Phy
    type5
}

/// Decodes control plane message from type5 to type1 bits
/// Returns (buf, bool) tuple
/// buf is BitBuffer with type1 bits if decoding successful
/// bool is true if CRC check was successful
pub fn decode_cp(lchan: LogicalChannel, prim: TpUnitdataInd, default_scramb_code: Option<u32>) -> (Option<BitBuffer>, bool) {

    assert!(lchan.is_control_channel() && lchan != LogicalChannel::Aach);

    // Various intermediate buffers, needed for decoding stages
    // We allocate the largest block we may possibly need
    let mut type4_arr = [0u8; MAX_TYPE345_BITS];
    let mut type3_arr = [0u8; MAX_TYPE345_BITS];
    let mut type3dp_arr = [0xFFu8; MAX_TYPE345_BITS*4];
    let mut type2_arr = [0u8; MAX_TYPE2_BITS];

    // Fetch decoding parameters for this logical channel type
    let params = errorcontrol_params::get_params(lchan);
    
    let mut type5 = prim.block;
    tracing::trace!("decode_cp {:?} type5: {:?}", lchan, type5.dump_bin());

    // Get scrambling code. For sync block, we use the default scranbling code. 
    // For others, we use the scrambling code previously retrieved from SYNC.
    let scrambling_code = if prim.block_type  == PhyBlockType::SB1 { 
        scrambler::SCRAMB_INIT
    } else if let Some(scrambling_code) = default_scramb_code { 
        scrambling_code
    } else {
        tracing::warn!("decode_cp: no scrambling code set, need to receive SYNC first");
        return (None, false);
    };
    
    scrambler::tetra_scramb_bits(scrambling_code, &mut type5);
    let mut type4 = type5;
    tracing::trace!("decode_cp {:?} type4: {:?}", lchan, type4.dump_bin());

    // De-interleaving, type4 -> type3
    type4.to_bitarr(&mut type4_arr[0..params.type345_bits]);
    interleaver::block_deinterleave(params.type345_bits, params.interleave_a, &type4_arr, &mut type3_arr);
    tracing::trace!("decode_cp {:?} type3: {:?}", lchan, BitBuffer::from_bitarr(&type3_arr[0..params.type345_bits]).dump_bin());

    // De-puncturing, type3 -> type3dp
    convenc::tetra_rcpc_depunct(RcpcPunctMode::Rate2_3, &type3_arr, params.type345_bits, &mut type3dp_arr);
    // tracing::trace!("decode_cp: t3dp:  {:?}", &type3dp_arr[0..4*params.type2_bits]);
    
    // Viterbi, type3dp -> type2    
    // viterbi_dec_sb1_wrapper(&type3dp_arr, &mut type2_arr, params.type2_bits);
    viterbi::dec_sb1(&type3dp_arr, &mut type2_arr, params.type2_bits);
    tracing::trace!("decode_cp {:?} type2: {:?}", lchan, BitBuffer::from_bitarr(&type2_arr[0..params.type2_bits]).dump_bin());
    
    // CRC check, type2 -> type1
    assert!(params.have_crc16);
    let type1_arr = &type2_arr[0..params.type1_bits];
    let crc = crc16::crc16_ccitt_bits(&type2_arr, params.type1_bits+16);
    let crc_ok = crc == crc16::TETRA_CRC_OK;
    let type1bits = BitBuffer::from_bitarr(&type1_arr[0..params.type1_bits]);
    // if crc_ok {
    //     // tracing::debug!("decode_cp {:>5?} crc: OK type1: {:?}", lchan, BitBuffer::from_bitarr(&type1_arr[0..params.type1_bits]).dump_bin());
    // } else {
    //     // tracing::info!("decode_cp {:>5?} CRC: WRONG {:x}", lchan, crc);
    // }
    
    (Some(type1bits), crc_ok)
}


/// Encodes traffic plane message from type1 to type5 bits
pub fn encode_tp(prim: TmvUnitdataReq, blk_num: u8) -> BitBuffer {

    // let lchan = prim.logical_channel;
    // assert!(lchan.is_control_channel() && lchan != LogicalChannel::Aach);

    // let params = errorcontrol_params::get_params(lchan);

    // assert!(prim.mac_block.get_len() == params.type1_bits, 
    //     "encode_cp: prim.mac_block length {} does not match type1_bits {} for lchan {:?}",
    //     prim.mac_block.get_len(), params.type1_bits, lchan);
    // tracing::trace!("encode_cp {:?} type1 {}", lchan, prim.mac_block.dump_bin());

    // // Convert bitbuffer to bitarray -> type1
    // prim.mac_block.seek(0);
    // let mut type2_arr = [0u8; 432]; // largest possible type1 block
    // prim.mac_block.to_bitarr(&mut type2_arr[0..params.type1_bits]);

    // // CRC addition, type1 -> type2
    // assert!(params.have_crc16);
    // let crc = !crc16::crc16_ccitt_bits( &type2_arr[0..params.type1_bits], params.type1_bits);
    // for i in 0..16 {
    //     type2_arr[params.type1_bits + i] = ((crc >> (15 - i)) & 1) as u8;
    // }
    // tracing::trace!("encode_cp {:?} type2: {:?}", lchan, BitBuffer::from_bitarr(&type2_arr[0..params.type2_bits]).dump_bin());

    // // Viterbi, type2 -> type3dp
    // let mut type3dp_arr = [0u8; 432*4];
    // let mut ces = ConvEncState::new();
    // ces.encode(&type2_arr[0..params.type2_bits], &mut type3dp_arr);
    // // tracing::trace!("encode_cp: t3dp:  {:?}", &type3dp_arr[0..4*params.type2_bits]);

    // // Puncturing, type3dp -> type3
    // let mut type3_arr = [0u8; 432]; // Need params.type345_bits
    // convenc::get_punctured_rate(RcpcPunctMode::Rate2_3, &type3dp_arr, &mut type3_arr);
    // tracing::trace!("encode_cp {:?} type3: {:?}", lchan, BitBuffer::from_bitarr(&type3_arr[0..params.type345_bits]).dump_bin());

    // // Interleaving, type3 -> type4
    // let mut type4_arr = [0u8; 432]; // Need params.type345_bits
    // interleaver::block_interleave(params.type345_bits, params.interleave_a, &type3_arr, &mut type4_arr);
    // let mut type4 = BitBuffer::from_bitarr(&type4_arr[0..params.type345_bits]);
    // tracing::trace!("encode_cp {:?} type4: {:?}", lchan, type4.dump_bin());

    // // Scrambling, type4 -> type5
    // scrambler::tetra_scramb_bits(prim.scrambling_code, &mut type4);
    // let type5 = type4;
    // tracing::trace!("encode_cp {:?} type5: {:?}", lchan, type5.dump_bin());

    // // Pass block to Phy
    // type5

    unimplemented_log!("encode_tp");
    if blk_num == 1 {

        // Full slot

        // Known sane block, which we inject for testing
        let mut type4 = BitBuffer::from_bitstr("101001111101001001110010101100101110100000000100111010101101001001000011110110111001001001110111011110001000100000100000100110010101001001000011011110110011010010010100000001111000010111010111010101100100010100110000011100001111010001000100111111110101000000010110110011100111000101000001110100100000101001011111111010000111101110111101101000100100111001010100100010100101000110010000101010011101000010101000111000111010010001100100");
        type4.seek(0);
        tracing::trace!("encode_tp type4: {:?}", type4.dump_bin());

        // Scrambling, type4 -> type5
        scrambler::tetra_scramb_bits(prim.scrambling_code, &mut type4);
        let type5 = type4;
        tracing::trace!("encode_tp type5: {:?}", type5.dump_bin());
        type5
        
    } else {
        // Half slot, first slot was apparently stolen
        unimplemented_log!("encode_tp: Half slot encoding not implemented yet");
        BitBuffer::new(MAX_TYPE345_HALFSLOT_BITS)
    }
}


/// Encodes AACH message from type1 to type5 bits
pub fn encode_aach(buf: BitBuffer, scrambling_code: u32) -> BitBuffer {
    
    let mut type1 = buf;
    tracing::trace!("encode_aach type1: {:?}", type1.dump_bin());
    assert!(type1.get_len_remaining() == 14);
    
    // RM code type1 -> type2
    let type1_int = type1.read_bits(14).unwrap() as u16; // Guaranteed
    let type2_int = rm3014::tetra_rm3014_compute(type1_int);

    let mut type2 = BitBuffer::new(30);
    type2.write_bits(type2_int as u64, 30);
    type2.seek(0);
    tracing::trace!("encode_aach type2: {:?}", type2.dump_bin());

    // No de-interleaving or rcpc needed for AACH

    // Scrambling, type2 -> type5
    scrambler::tetra_scramb_bits(scrambling_code, &mut type2);
    let type5 = type2;

    tracing::trace!("encode_aach type5: {:?}", type5.dump_bin());

    type5
}

/// Decodes 
pub fn decode_aach(buf: BitBuffer, scrambling_code: u32) -> BitBuffer {

    let mut type5 = buf;
    tracing::trace!("decode_aach type5: {:?}", type5.dump_bin());
    assert!(type5.get_len_remaining() == 30);

    // Unscrambling, type5 -> type2
    scrambler::tetra_scramb_bits(scrambling_code, &mut type5);
    let mut type2 = type5;
    tracing::trace!("decode_aach type2: {:?}", type2.dump_bin());

    // No de-interleaving or rcpc needed for AACH

    // RM code type2 -> type1
    
    // Convert to int and perform single-bit error correction
    // TODO FIXME: Multi-bit error correction (Clause 8.3.1.1)
    let x = type2.read_bits(30).unwrap() as u32; // Guaranteed
    let y = rm3014::tetra_rm3014_decode_limited_ecc(x);

    // Write error-corrected data to type1 and return
    let mut type1 = type2;
    type1.set_raw_start(0);
    type1.set_raw_pos(0);
    type1.set_raw_end(14);
    type1.write_bits(y as u64, 14);
    type1.seek(0);
    
    tracing::debug!("decode_aach type1: {:?}", type1.dump_bin());
    type1
}

#[cfg(test)]
mod tests {
    use tetra_core::{BurstType, PhyBlockNum, TrainingSequence, debug::setup_logging_verbose};

    use super::*;

    /// Tests SCH/HD, STCH, BNCH encoding and decoding
    #[test]
    fn test_encdec_bnch() {

        // setup_logging_verbose();
        let type1vec = "1000001111101001010000000000101001101110011000000000000000001010000101010100000000000000000000101111111111111111110100100000";
        let type5vec = "001101111110011111000110100001101110011100110000111100011000011100101011111100010101101001101001001110011100001010001101101010100000000011010001001101001010101100100110011001111100001011000001010010000011010110110110";
        let bb = BitBuffer::from_bitstr(type1vec);
        let lchan = LogicalChannel::Bnch;
        let scramb_code = scrambler::tetra_scramb_get_init(204, 1337, 1);
        // println!("start: {}", bb.dump_bin());
        let prim_req = TmvUnitdataReq { 
            mac_block: bb, 
            logical_channel: lchan, 
            scrambling_code: scramb_code 
        };
        let type5 = encode_cp(prim_req);
        // println!("type5:   {}", type5.dump_bin());
        assert_eq!(type5vec, type5.to_bitstr());
        
        let prim_ind = TpUnitdataInd { 
            train_type: TrainingSequence::SyncTrainSeq, 
            burst_type: BurstType::SDB, 
            block_type: PhyBlockType::SB2, 
            block_num: PhyBlockNum::Block2, 
            block: type5
        };

        let (type1, crc_ok) = decode_cp(lchan, prim_ind, Some(scramb_code));
        let type1 = type1.unwrap();
        
        assert!(crc_ok);    
        assert_eq!(type1vec, type1.to_bitstr());
    }

    /// Tests BSCH encoding and decoding
    #[test]
    fn test_encdec_bsch() {

        // setup_logging_verbose();
        let type1_vec = "000100000111000010000010000000000110011000001010011100110001";
        let bb = BitBuffer::from_bitstr(type1_vec);
        let lchan = LogicalChannel::Bsch;
        let scramb_code = scrambler::SCRAMB_INIT;
        let prim_req = TmvUnitdataReq { 
            mac_block: bb, 
            logical_channel: lchan, 
            scrambling_code: scramb_code 
        };
        let type5 = encode_cp(prim_req);
        let prim_ind = TpUnitdataInd { 
            train_type: TrainingSequence::SyncTrainSeq, 
            burst_type: BurstType::SDB, 
            block_type: PhyBlockType::SB2, 
            block_num: PhyBlockNum::Block2, 
            block: type5
        };

        let (type1, crc_ok) = decode_cp(lchan, prim_ind, Some(scramb_code));
        let type1 = type1.unwrap();
        assert!(crc_ok);            
        assert_eq!(type1_vec, type1.to_bitstr());
    }

    /// Tests AACH encoding and decoding
    #[test]
    fn test_encdec_aach() {
        // setup_logging_verbose();
        let scramb_code = scrambler::tetra_scramb_get_init(204, 1337, 1);
        let type5vec = "100100100001011110111010111011";
        let type1vec = "00001010001010";
        
        let type5vec_bb = BitBuffer::from_bitstr(type5vec);
        let type1vec_bb = BitBuffer::from_bitstr(type1vec);

        let type1 = decode_aach(type5vec_bb, scramb_code);
        let type5 = encode_aach(type1vec_bb, scramb_code);
        
        assert_eq!(type5vec, type5.to_bitstr());
        assert_eq!(type1vec, type1.to_bitstr());
    }

    /// Tests SCH/F encoding and decoding
    #[test]
    fn test_encdec_sch_f() {

        setup_logging_verbose();
        let type1_vec = "0000000000110001000000000010011100010001000001110010000010000001000000000010011100010001010000000000001000110110011011100000100110000001011100000000110101000110011100000100000000000000000100001000000000000000000000000000000000000000000000000000000000000000000000000000";
        let bb = BitBuffer::from_bitstr(type1_vec);
        let lchan = LogicalChannel::SchF;
        let scramb_code = scrambler::tetra_scramb_get_init(204, 1337, 1);
        let prim_req = TmvUnitdataReq { 
            mac_block: bb, 
            logical_channel: lchan, 
            scrambling_code: scramb_code 
        };
        let type5 = encode_cp(prim_req);
        let prim_ind = TpUnitdataInd { 
            train_type: TrainingSequence::NormalTrainSeq1, 
            burst_type: BurstType::NDB, 
            block_type: PhyBlockType::NDB, 
            block_num: PhyBlockNum::Both, 
            block: type5
        };

        let (type1, crc_ok) = decode_cp(lchan, prim_ind, Some(scramb_code));
        let type1 = type1.unwrap();
        assert!(crc_ok);            
        assert_eq!(type1_vec, type1.to_bitstr());
    }
}
