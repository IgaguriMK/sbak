//! ツールのバージョン情報を生成する補助関数群

/// Gitのハッシュ値の16進数表現の長さ(40文字)
pub const GIT_HASH_LEN: usize = 40;

/// コンパイル時の環境変数に設定されたGitリポジトリ情報から、[Semantic Versioning 2.0.0](https://semver.org/)準拠のバージョン表記を生成する。
///
/// 必要な環境変数は以下の通り。
///
/// | 変数名 | 説明 |
/// |:--:|:---|
/// | `GIT_BRANCH`    | カレントブランチ名 |
/// | `GIT_HASH`      | コミットのハッシュ値 |
/// | `GIT_DIFF`      | `git diff` の行数 |
/// | `GIT_UNTRACKED` | Untrackedなファイルの有無 (0 = False, 1 = True) |
///
/// 出力のハッシュ値の長さは`hash_len`で指定する。
/// 0を指定した場合、ハッシュ値は付加されない。
pub fn version(hash_len: usize) -> String {
    let base_ver = env!("CARGO_PKG_VERSION");
    let branch = option_env!("GIT_BRANCH").unwrap_or("unknown");
    let hash = option_env!("GIT_HASH");
    let diff = if let Some(lines) = option_env!("GIT_DIFF") {
        lines != "0"
    } else {
        false
    };
    let untracked = if let Some(lines) = option_env!("GIT_UNTRACKED") {
        lines != "0"
    } else {
        false
    };

    let mut ver = base_ver.to_string();

    let mut pre_release = String::new();
    pre_release.push_str(branch);
    if diff | untracked {
        pre_release.push_str(".uncommitted");
    }

    if pre_release != "master" {
        ver.push_str("-");
        ver.push_str(&pre_release);
    }

    if 0 < hash_len {
        if let Some(hash) = hash {
            ver.push_str("+");
            if hash_len < hash.len() {
                ver.push_str(&hash[..hash_len]);
            } else {
                ver.push_str(hash);
            }
        }
    }

    ver
}
