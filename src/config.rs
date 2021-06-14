//! 設定ファイルを扱う。

use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use toml::to_string_pretty;

/// 設定ファイルの内容
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    repository_path: Option<PathBuf>,
    #[serde(default)]
    log: Log,
}

impl Config {
    /// リポジトリのパスを取得する。
    pub fn repository_path(&self) -> Option<&Path> {
        self.repository_path.as_ref().map(|p| p.as_ref())
    }

    /// リポジトリのパスを設定する。
    pub fn set_repository_path<P: AsRef<Path>>(&mut self, path: P) {
        self.repository_path = Some(path.as_ref().to_owned());
    }

    /// ログ表示のレベルを設定する。
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log.level = Some(level);
    }

    /// 文字列で指定されたログ表示のレベルを設定する。
    pub fn set_log_level_str(&mut self, level_str: &str) -> Result<(), LogLevelParseError> {
        self.set_log_level(level_str.parse()?);
        Ok(())
    }

    /***********************************************************/

    /// 他の設定ファイルの設定値で上書きした新規の`Config`を返す。
    pub fn merged(&self, overwrite: &Config) -> Config {
        Config {
            repository_path: merge(&self.repository_path, &overwrite.repository_path),
            log: self.log.merged(&overwrite.log),
        }
    }

    /// 設定値を標準出力に表示する
    pub fn show(&self) {
        let s = to_string_pretty(self).unwrap();
        print!("{}", s);
    }
}

fn merge<T: Clone>(x: &Option<T>, overwrite: &Option<T>) -> Option<T> {
    overwrite.clone().or_else(|| x.clone())
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Log {
    output: Option<String>,
    level: Option<LogLevel>,
}

impl Log {
    pub fn merged(&self, overwrite: &Log) -> Log {
        Log {
            output: merge(&self.output, &overwrite.output),
            level: merge(&self.level, &overwrite.level),
        }
    }
}

/// ログ表示のレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// 無効
    Off,
    /// エラーのみ
    Error,
    /// 警告を表示
    Warn,
    /// 詳細動作を表示
    Info,
    /// デバッグ用
    Debug,
}

impl Default for LogLevel {
    fn default() -> LogLevel {
        LogLevel::Warn
    }
}

impl FromStr for LogLevel {
    type Err = LogLevelParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "off" => Ok(LogLevel::Off),
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            s => Err(LogLevelParseError(s.to_string())),
        }
    }
}

/// ログレベルのパースに失敗したときに返されるエラー
#[derive(Debug, thiserror::Error)]
#[error("invalid log level: {0}")]
pub struct LogLevelParseError(String);
