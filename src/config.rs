//! 設定ファイルを扱う。

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(target_os = "windows")]
use std::env;

use anyhow::{Context, Error, Result};
use log::{error, LevelFilter};
use serde::{Deserialize, Serialize};
use toml::de::from_slice;
use toml::to_string_pretty;

use crate::smalllog;

/// 指定パスから設定ファイルを読み込む
pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
    let mut f = File::open(&path).context("opening config file")?;
    let mut buf = Vec::<u8>::new();
    f.read_to_end(&mut buf)?;

    Ok(from_slice(&buf)?)
}

/// 指定パスからの設定ファイルの読み込みを試行する。
///
/// 存在しない場合Noneを返す。
pub fn try_load<P: AsRef<Path>>(path: P) -> Result<Option<Config>> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(None);
    }

    Some(load(path)).transpose()
}

/// 既定のパス([`config_pathes()`](fn.config_pathes.html))から設定を読み込む
pub fn auto_load() -> Result<Config> {
    let mut config = Config::default();

    for path in config_pathes()? {
        if let Some(c) = try_load(&path)? {
            config = config.merged(&c);
        }
    }

    Ok(config)
}

/// 起動時に読み込む設定ファイルの探索パスの一覧を返す。
///
/// ターゲットとなる環境に応じて切り替えられる。
/// 現在表示されているのはLinux向け。
#[cfg(target_os = "linux")]
pub fn config_pathes() -> Result<Vec<PathBuf>> {
    let mut pathes = Vec::<PathBuf>::new();

    // システム共通設定
    pathes.push("/etc/sbak.toml".parse().unwrap());

    // ユーザー設定
    if let Some(mut home_dir) = dirs::home_dir() {
        home_dir.push(".sbak.toml");
        pathes.push(home_dir);
    }

    Ok(pathes)
}

/// 起動時に読み込む設定ファイルの探索パスの一覧を返す。
///
/// ターゲットとなる環境に応じて切り替えられる。
/// 現在表示されているのはWindows向け。
#[cfg(target_os = "windows")]
pub fn config_pathes() -> Result<Vec<PathBuf>> {
    let mut pathes = Vec::<PathBuf>::new();

    // インストール場所設定
    if let Ok(exe_path) = env::current_exe() {
        let mut exe_dir = exe_path.canonicalize()?.parent().unwrap().to_owned();
        exe_dir.push("sbak.toml");
        pathes.push(exe_dir);
    }

    // ユーザー設定（UNIXスタイルのパス）
    if let Some(home_dir) = dirs::home_dir() {
        let mut with_dot = home_dir.clone();
        with_dot.push(".sbak.toml");
        pathes.push(with_dot);

        let mut without_dot = home_dir;
        without_dot.push("sbak.toml");
        pathes.push(without_dot);
    }

    // ユーザー設定（Windowsスタイルのパス）
    if let Some(mut roaming) = dirs::config_dir() {
        roaming.push("sbak");
        roaming.push("config.toml");
        pathes.push(roaming);
    }

    Ok(pathes)
}

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
    pub fn set_log_level_str(&mut self, level_str: &str) -> Result<()> {
        self.set_log_level(level_str.parse()?);
        Ok(())
    }

    /// ログ設定をロガーに適用する。
    pub fn apply_log(&self) {
        self.log.apply();
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
    fn apply(&self) {
        match self.output.as_deref().unwrap_or("stderr") {
            "stderr" => smalllog::use_stderr(),
            name => {
                if let Err(e) = smalllog::use_file(name) {
                    error!("can't open log file {}: {}", name, e);
                }
            }
        }

        smalllog::set_level(self.level.unwrap_or_default().into());
    }

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
    /// より詳細なデバッグ用
    Trace,
}

impl Default for LogLevel {
    fn default() -> LogLevel {
        LogLevel::Warn
    }
}

impl FromStr for LogLevel {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "off" => Ok(LogLevel::Off),
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            s => Err(Error::msg(format!("Invalid log level: {}", s))),
        }
    }
}

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}
