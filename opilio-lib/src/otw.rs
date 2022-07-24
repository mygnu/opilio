use defmt::Format;
use heapless::Vec;
use postcard::{from_bytes, to_vec};
use serde::Serialize;

use crate::{error::Error, Cmd, Data, DataRef, Result, MAX_SERIAL_DATA_SIZE};

/// Over The Wire protocol
#[derive(Debug, Format, PartialEq)]
pub struct OTW {
    pub cmd: Cmd,
    pub data: Data,
}

impl OTW {
    pub fn serialised_vec(
        cmd: Cmd,
        data: DataRef,
    ) -> Result<Vec<u8, MAX_SERIAL_DATA_SIZE>> {
        #[derive(Serialize)]
        struct OtwSerial<'a> {
            cmd: Cmd,
            data: DataRef<'a>,
        }

        if match cmd {
            Cmd::GetStats | Cmd::SaveConfig | Cmd::GetConfig => {
                matches!(data, DataRef::Empty)
            }
            Cmd::UploadSetting => {
                matches!(data, DataRef::Setting(_))
            }
            Cmd::Config => {
                matches!(data, DataRef::Config(_))
            }
            Cmd::Result => matches!(data, DataRef::Result(_)),
            Cmd::Stats => matches!(data, DataRef::Stats(_)),
            Cmd::UploadGeneral => matches!(data, DataRef::General(_)),
        } {
            let s = OtwSerial { cmd, data };
            to_vec(&s).map_err(Error::from)
        } else {
            Err(Error::InvalidCmdDataPair)
        }
    }

    pub fn serialised_ok() -> &'static [u8; 3] {
        &[6_u8, 4, 0]
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self> {
        if slice.len() < 2 {
            return Err(Error::Deserialize);
        }
        let command = from_bytes(&slice[0..2])?;

        let data = match command {
            Cmd::UploadSetting => Data::Setting(from_bytes(&slice[2..])?),
            Cmd::Config => Data::Config(from_bytes(&slice[2..])?),
            Cmd::Stats => Data::Stats(from_bytes(&slice[2..])?),
            Cmd::GetConfig => Data::SettingId(from_bytes(&slice[2..])?),
            Cmd::Result => Data::Result(from_bytes(&slice[2..])?),
            Cmd::UploadGeneral => Data::General(from_bytes(&slice[2..])?),
            Cmd::GetStats | Cmd::SaveConfig => Data::Empty,
        };
        Ok(Self { cmd: command, data })
    }
}
