use defmt::Format;
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use serde::Serialize;

use crate::{error::Error, Cmd, Data, Result, MAX_SERIAL_DATA_SIZE};

/// Over The Wire protocol
#[derive(Debug, Format, Serialize, PartialEq)]
pub struct OTW {
    cmd: Cmd,
    data: Data,
}

impl OTW {
    pub fn new(cmd: Cmd, data: Data) -> Result<Self> {
        if match cmd {
            Cmd::GetStats | Cmd::SaveConfig | Cmd::GetConfig => {
                matches!(data, Data::Empty)
            }
            Cmd::Config | Cmd::SetConfig => {
                matches!(data, Data::Config(_))
            }
            Cmd::Result => matches!(data, Data::Result(_)),
            Cmd::Stats => matches!(data, Data::Stats(_)),
            Cmd::SetStandby => matches!(data, Data::U64(_)),
        } {
            Ok(Self { cmd, data })
        } else {
            Err(Error::InvalidCmdDataPair)
        }
    }

    pub fn data(self) -> Data {
        self.data
    }
    pub fn cmd(&self) -> Cmd {
        self.cmd
    }

    pub fn to_vec(&self) -> Result<Vec<u8, MAX_SERIAL_DATA_SIZE>> {
        to_vec(&self).map_err(Error::from)
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self> {
        if slice.len() < 2 {
            return Err(Error::Deserialize);
        }
        let command = from_bytes(&slice[0..2])?;

        let data = match command {
            Cmd::Config | Cmd::SetConfig => {
                Data::Config(from_bytes(&slice[2..])?)
            }
            Cmd::Stats => Data::Stats(from_bytes(&slice[2..])?),
            Cmd::GetConfig => Data::FanId(from_bytes(&slice[2..])?),
            Cmd::Result => Data::Result(from_bytes(&slice[2..])?),
            Cmd::SetStandby => Data::U64(from_bytes(&slice[2..])?),
            Cmd::GetStats | Cmd::SaveConfig => Data::Empty,
        };
        Ok(Self { cmd: command, data })
    }
}
