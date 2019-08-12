//! 簡易ロガー

use std::env::var;
use std::fs::File;
use std::io::{self, stderr, Stderr, Write};
use std::path::Path;
use std::sync::Mutex;

use lazy_static::{initialize, lazy_static};
use log::{set_logger, set_max_level, warn, LevelFilter, Log, Metadata, Record};

lazy_static! {
    static ref STATE: Mutex<LoggerState> = {
        Mutex::new(LoggerState {
            out: Out::Stderr(stderr()),
            level: LevelFilter::Trace,
            show_detail_level: LevelFilter::Debug,
        })
    };
}

const LOGGER: Logger = Logger {};

/// ロガーを初期化する。
///
/// 環境変数`sbak_log`が設定されている場合、その指定レベルに設定する。
pub fn init() {
    initialize(&STATE);
    set_logger(&LOGGER).unwrap();
    set_max_level(LevelFilter::Trace);

    if let Ok(sbak_log) = var("SBAK_LOG") {
        match sbak_log.to_ascii_lowercase().as_str() {
            "off" => set_level(LevelFilter::Off),
            "error" => set_level(LevelFilter::Error),
            "warn" => set_level(LevelFilter::Warn),
            "info" => set_level(LevelFilter::Info),
            "debug" => set_level(LevelFilter::Debug),
            "trace" => set_level(LevelFilter::Trace),
            s => warn!("unknown log level SBAK_LOG={}", s),
        }
    }
}

/// ログ出力先を標準エラー出力にする。
pub fn use_stderr() {
    let mut state = STATE.lock().unwrap();
    state.out = Out::Stderr(stderr());
}

/// ログ出力先を標準エラー出力にする。
///
/// # Failures
/// ファイルのオープンに失敗した場合、エラーを返す。
pub fn use_file<P: AsRef<Path>>(path: P) -> Result<(), io::Error> {
    let f = File::create(&path)?;
    let mut state = STATE.lock().unwrap();
    state.out = Out::File(f);
    Ok(())
}

/// ログ出力のレベルを設定する。
///
/// デフォルト値は`LevelFilter::Trace`。
pub fn set_level(level: LevelFilter) {
    let mut state = STATE.lock().unwrap();
    state.level = level;
}

/// ログ出力により詳細な情報を表示するレベルを設定する。
///
/// デフォルト値は`LevelFilter::Debug`。
pub fn set_show_detail_level(level: LevelFilter) {
    let mut state = STATE.lock().unwrap();
    state.show_detail_level = level;
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let state = STATE.lock().unwrap();
        metadata.level() <= state.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut state = STATE.lock().unwrap();
        let show_detail = record.level() >= state.show_detail_level;
        let w = state.out.writer();

        if show_detail {
            writeln!(
                w,
                "[ {:5} ] {}:{} : {}",
                record.level(),
                record.module_path().unwrap_or("<unknown module>"),
                record
                    .line()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "??".to_owned()),
                record.args()
            )
            .unwrap();
        } else {
            writeln!(
                w,
                "[ {:5} ] {} : {}",
                record.level(),
                record.module_path().unwrap_or("<unknown module>"),
                record.args()
            )
            .unwrap();
        }
    }

    fn flush(&self) {
        let mut state = STATE.lock().unwrap();
        let w = state.out.writer();
        w.flush().unwrap();
    }
}

struct LoggerState {
    out: Out,
    level: LevelFilter,
    show_detail_level: LevelFilter,
}

enum Out {
    Stderr(Stderr),
    File(File),
}

impl Out {
    fn writer(&mut self) -> &mut dyn Write {
        match self {
            Out::Stderr(ref mut w) => w,
            Out::File(ref mut w) => w,
        }
    }
}
