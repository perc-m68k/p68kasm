pub mod addressing;
use addressing::*;

pub enum WordSize {
    W,
    L,
}

pub enum IntSize {
    B,
    W,
    L,
}

pub enum SmallSize {
    B,
    W,
}

pub enum Instruction {
    LEA(LeaFirstArg, An),
    LINK(An, ImmediateData),
    MOVE(IntSize, SrcEA, DstEA),
    MOVEA(WordSize, SrcEA, An),
}
