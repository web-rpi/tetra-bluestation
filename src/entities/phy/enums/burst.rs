#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PhyBlockType {
    BBK,
    /// TODO FIXME Merge SB1 and SB2 into SDB
    SB1,
    SB2,
    NDB,

    NUB,
    SSN1,
    SSN2
}

/// Clause 9.4.4.1
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BurstType {
    /// Control Uplink Burst
    CUB,

    /// Normal Uplink Burst
    NUB,

    /// Normal Downlink Burst. We don't differentiate between continuous and discontinuous bursts
    NDB, 
    /// Syncrhonization Downlink Burst. We don't differentiate between continuous and discontinuous bursts
    SDB, 
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PhyBlockNum {
    Both,
    Block1,
    Block2,
    Undefined
}

/// Training sequences
#[derive(Debug, Copy, Clone, PartialEq)]
#[derive(Default)]
pub enum TrainingSequence {
    /// 22 n bits
    NormalTrainSeq1 = 1,
    /// 22 p bits
    NormalTrainSeq2 = 2,
    /// 22 q bits
    NormalTrainSeq3 = 3,
    /// 30 x bits
    ExtendedTrainSeq = 4,
    /// 38 y bits
    SyncTrainSeq = 5,
    /// Not found
    #[default]
    NotFound = 0,
}


