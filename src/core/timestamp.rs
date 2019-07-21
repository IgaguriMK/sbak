use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use std::convert::TryFrom;
use std::fmt;

use serde::{Serialize, Deserialize};
use failure::Fail;

/// ファイルの更新日時
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Result<Timestamp> {
        Timestamp::try_from(SystemTime::now())
    }
}

impl TryFrom<SystemTime> for Timestamp {
    type Error = Error;

    fn try_from(t: SystemTime) -> Result<Timestamp> {
        let unix_time_u64 = t.duration_since(UNIX_EPOCH)?.as_secs();
        Ok(Timestamp(unix_time_u64))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "timestamp is older than UNIX epoch")]
    Timestamp,
}

impl From<SystemTimeError> for Error {
    fn from(_e: SystemTimeError) -> Error {
        Error::Timestamp
    }
}
