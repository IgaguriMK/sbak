//! ツールのバージョン情報を生成する補助関数群

/// Gitのハッシュ値の16進数表現の長さ(40文字)
pub const GIT_HASH_LEN: usize = 40;

/// バージョンを取得する。
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
