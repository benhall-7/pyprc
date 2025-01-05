use crate::{Hash, ParamList2, ParamStruct2, ParamType};

/// A fake clone implementation. PyO3 uses blanket implementations of FromPyObject for pyclasses
/// when they implement Clone. I need to have custom implementations for some types
pub(crate) trait Duplicate: Sized {
    fn duplicate(&self) -> Self;
}

impl<T> Duplicate for T
where
    T: Clone,
{
    fn duplicate(&self) -> Self {
        self.clone()
    }
}

impl Duplicate for ParamType {
    fn duplicate(&self) -> Self {
        match self {
            ParamType::Bool(v) => ParamType::Bool(*v),
            ParamType::I8(v) => ParamType::I8(*v),
            ParamType::U8(v) => ParamType::U8(*v),
            ParamType::I16(v) => ParamType::I16(*v),
            ParamType::U16(v) => ParamType::U16(*v),
            ParamType::I32(v) => ParamType::I32(*v),
            ParamType::U32(v) => ParamType::U32(*v),
            ParamType::Float(v) => ParamType::Float(*v),
            ParamType::Hash(v) => ParamType::Hash(v.duplicate()),
            ParamType::Str(v) => ParamType::Str(v.clone()),
            ParamType::List(v) => ParamType::List(v.duplicate()),
            ParamType::Struct(v) => ParamType::Struct(v.duplicate()),
        }
    }
}

impl Duplicate for Hash {
    fn duplicate(&self) -> Self {
        Hash { inner: self.inner }
    }
}

impl Duplicate for ParamList2 {
    fn duplicate(&self) -> Self {
        ParamList2(self.0.duplicate())
    }
}

impl Duplicate for ParamStruct2 {
    fn duplicate(&self) -> Self {
        ParamStruct2(
            self.0
                .iter()
                .map(|(h, p)| (h.duplicate(), p.duplicate()))
                .collect(),
        )
    }
}
