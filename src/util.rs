//! 他のクレートとの接続用などのユーティリティ集。

pub mod time;

use std::error::Error;

use log::error;

pub(crate) fn dump_error(e: impl Error) {
    error!("{}", e);
    dump_sources(e.source());
}

fn dump_sources(e: Option<&(dyn Error + 'static)>) {
    if let Some(e) = e {
        error!("    # {}", e);
        dump_sources(e.source());
    }
}
