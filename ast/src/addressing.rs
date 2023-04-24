#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct An(u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dn(u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Indirect(An);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndirectPostInc(An);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndirectPreDec(An);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndirectDisp(u16, An);

// TODO 2.2.7
// TODO 2.2.8
// TODO 2.2.9
// TODO 2.2.10
// TODO 2.2.11
// TODO 2.2.12
// TODO 2.2.13
// TODO 2.2.14
// TODO 2.2.15

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsoluteShort(u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsoluteLong(u32);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImmediateData(Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SrcEA {
    Dn(Dn),
    An(An),
    Indirect(Indirect),
    IndirectDisp(IndirectDisp),
    IndirectPostInc(IndirectPostInc),
    IndirectPreDec(IndirectPreDec),
    ImmediateData(ImmediateData),
    AbsoluteLong(AbsoluteLong),
    AbsoluteShort(AbsoluteShort),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DstEA {
    Dn(Dn),
    Indirect(Indirect),
    IndirectDisp(IndirectDisp),
    IndirectPostInc(IndirectPostInc),
    IndirectPreDec(IndirectPreDec),
    AbsoluteLong(AbsoluteLong),
    AbsoluteShort(AbsoluteShort),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LeaFirstArg {
    Indirect(Indirect),
    IndirectDisp(IndirectDisp),
    AbsoluteLong(AbsoluteLong),
    AbsoluteShort(AbsoluteShort),
}
