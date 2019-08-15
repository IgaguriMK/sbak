//! `chrono`及び`chrono-tz`の`TimeZone`を一括して扱う。

use std::fmt;

use chrono::{DateTime, Local, TimeZone, Utc};
use chrono_tz::Tz as ChronoTz;

const FORMAT_DATETIME: &str = "%Y-%m-%d %H:%M:%S";

/// 画面出力で使用するタイムゾーンを表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tz {
    /// UTC
    Utc,
    /// 実行環境のタイムゾーン
    Local,
    /// 特定のタイムゾーン
    Tz(ChronoTz),
}

impl Tz {
    /// 名前からタイムゾーンを得る。
    ///
    /// `name`が`None`のときは`Tz::Local`を返す。
    ///
    /// `local`, `Local`, `LOCAL` 等は`Tz::Local`に、`utc`, `Utc`, `UTC`等は`Tz::Utc`に変換される。
    ///
    /// 残りは全てIANA Time Zone Databaseのタイムゾーン名として解釈される。
    ///
    /// # Failures
    ///
    /// 名前が無効な場合、その無効な名前をコピーした`String`を返す。
    pub fn from_name(name: Option<&str>) -> Result<Tz, String> {
        if name == None {
            return Ok(Tz::Local);
        }

        let name = name.unwrap();

        if &name.to_ascii_lowercase() == "local" {
            return Ok(Tz::Local);
        }

        if &name.to_ascii_lowercase() == "utc" {
            return Ok(Tz::Utc);
        }

        name.parse().map(Tz::Tz)
    }

    /// 指定された`unix_epoch`の日時表現`OutputDateTime`を返す。
    pub fn at(self, unix_epoch: u64) -> OutputDateTime {
        OutputDateTime {
            unix_epoch,
            zone: self,
        }
    }
}

/// 出力に使われる日時表現
#[derive(Debug, Clone)]
pub struct OutputDateTime {
    unix_epoch: u64,
    zone: Tz,
}

impl OutputDateTime {
    /// `yyyy-mm-dd HH:MM:SS`形式でフォーマットした文字列を返す。
    pub fn datetime_string(&self) -> String {
        self.format_datetime().to_string()
    }

    /// `yyyy-mm-dd HH:MM:SS`形式でフォーマットした結果を表示する、`Display`を実装した内部型を返す。
    pub fn format_datetime(&self) -> impl fmt::Display {
        match self.zone {
            Tz::Utc => self.datetime_in(&Utc).format(FORMAT_DATETIME),
            Tz::Local => self.datetime_in(&Local).format(FORMAT_DATETIME),
            Tz::Tz(ref tz) => self.datetime_in(tz).format(FORMAT_DATETIME),
        }
    }

    fn datetime_in<Z: TimeZone>(&self, zone: &Z) -> DateTime<Z> {
        zone.timestamp(self.unix_epoch as i64, 0)
    }
}

impl fmt::Display for OutputDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format_datetime())
    }
}
