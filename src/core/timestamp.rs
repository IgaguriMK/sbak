//! タイムスタンプ

use std::convert::TryFrom;
use std::fmt;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

use failure::Fail;
use serde::{Deserialize, Serialize};

/// 秒精度のタイムスタンプ
/// 
/// [`std::time::SystemTime`](https://doc.rust-lang.org/std/time/struct.SystemTime.html) 由来の時刻をUNIX epochからの経過秒数で管理する。
/// UNIX epochより古い時刻には対応していない。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    /// 現在時刻を取得する
    /// 
    /// # Failures
    /// 
    /// システムの現在時刻がUNIX epoch (`1970-01-01 00:00:00 UTC`) より前である場合、 [`Error::NegativeUnixTime`](enum.Error.html) を返す。
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
    /// 対象日時のUNIX epochが負になっている
    #[fail(display = "timestamp is older than UNIX epoch")]
    NegativeUnixTime,
}

impl From<SystemTimeError> for Error {
    fn from(_e: SystemTimeError) -> Error {
        Error::NegativeUnixTime
    }
}
