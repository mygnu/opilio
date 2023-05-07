use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    Deserialize,
    FlashErase,
    FlashRead,
    FlashWrite,
    SerialRead,
    SerialWrite,
    Serialize,
    InvalidMsgDataPair,
    Unknown,
    TempRead,
}

impl From<postcard::Error> for Error {
    fn from(e: postcard::Error) -> Self {
        use postcard::Error::*;
        match e {
            DeserializeBadBool
            | DeserializeBadChar
            | DeserializeBadEncoding
            | DeserializeBadEnum
            | DeserializeBadOption
            | DeserializeBadUtf8
            | DeserializeBadVarint
            | DeserializeUnexpectedEnd
            | SerdeDeCustom => Self::Deserialize,
            SerdeSerCustom
            | SerializeBufferFull
            | SerializeSeqLengthUnknown => Self::Serialize,
            _ => Self::Unknown,
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    extern crate std;
    use std::{error::Error, fmt::Display};

    impl Display for super::Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl Error for super::Error {}
}
