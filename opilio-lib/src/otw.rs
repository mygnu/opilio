use heapless::Vec;
use postcard::{from_bytes, to_vec};
use serde::Serialize;

use crate::{error::Error, Data, DataRef, Msg, Result, MAX_SERIAL_DATA_SIZE};

/// Over The Wire protocol
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OTW {
    pub msg: Msg,
    pub data: Data,
}

impl OTW {
    pub fn serialised_vec(
        msg: Msg,
        data: DataRef,
    ) -> Result<Vec<u8, MAX_SERIAL_DATA_SIZE>> {
        #[derive(Serialize)]
        struct OtwSerial<'a> {
            msg: Msg,
            data: DataRef<'a>,
        }

        if match msg {
            Msg::GetStats
            | Msg::SaveConfig
            | Msg::GetConfig
            | Msg::Reload
            | Msg::Ping => {
                matches!(data, DataRef::Empty)
            }
            Msg::Config => {
                matches!(data, DataRef::Config(_))
            }
            Msg::UploadConfig => {
                matches!(data, DataRef::Config(_))
            }
            Msg::Result => matches!(data, DataRef::Result(_)),
            Msg::Stats => matches!(data, DataRef::Stats(_)),
            Msg::Pong => matches!(data, DataRef::Pong(_)),
        } {
            let s = OtwSerial { msg, data };
            to_vec(&s).map_err(Error::from)
        } else {
            Err(Error::InvalidMsgDataPair)
        }
    }

    // returns Result<
    pub fn serialised_ok() -> &'static [u8; 3] {
        &[6_u8, 4, 0]
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self> {
        if slice.len() < 2 {
            return Err(Error::Deserialize);
        }
        let command = from_bytes(&slice[0..2])?;

        let data = match command {
            Msg::Config | Msg::UploadConfig => {
                Data::Config(from_bytes(&slice[2..])?)
            }
            Msg::Stats => Data::Stats(from_bytes(&slice[2..])?),
            Msg::Result => Data::Result(from_bytes(&slice[2..])?),
            Msg::Pong => Data::Pong(from_bytes(&slice[2..])?),

            Msg::Ping
            | Msg::GetConfig
            | Msg::GetStats
            | Msg::SaveConfig
            | Msg::Reload => Data::Empty,
        };
        Ok(Self { msg: command, data })
    }
}
