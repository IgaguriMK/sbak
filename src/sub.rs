//! サブコマンドの基盤部分

use std::collections::BTreeMap;
use std::fmt;

use clap::{App, ArgMatches};

use crate::config::Config;

mod backup;
mod history;
mod info;
mod init;

/// サブコマンドを表現するトレイト
pub trait SubCmd {
    /// サブコマンドの名前を返す。
    fn name(&self) -> &str;

    /// コマンドライン引数の定義を返す。
    fn command_args(&self) -> App;

    /// サブコマンドを実行する。
    ///
    /// 完了後は制御を返さず、 [`std::process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html) などでプロセスを終了する。
    fn exec(&self, matches: &ArgMatches, config: Config) -> !;
}

/// 組み込まれているサブコマンドすべてを含む [`SubCommandSet`](struct.SubCommandSet.html) を返す。
pub fn sub_commands() -> SubCommandSet {
    let mut set = SubCommandSet::new();

    set.append(backup::new());
    set.append(history::new());
    set.append(init::new());
    set.append(info::new());

    set
}

/// サブコマンドの一覧を表現する
#[derive(Default)]
pub struct SubCommandSet {
    table: BTreeMap<String, Box<dyn SubCmd>>,
}

impl<'a> SubCommandSet {
    /// 空の一覧を返す。
    pub fn new() -> SubCommandSet {
        SubCommandSet {
            table: BTreeMap::new(),
        }
    }

    /// clapで使用するサブコマンドの定義リストを返す。
    pub fn arg_defs(&'a self) -> impl Iterator<Item = App<'a, 'a>> {
        self.table.iter().map(|(_, c)| c.command_args())
    }

    /// サブコマンド `name` を起動する。
    ///
    /// `name` が一致したサブコマンドがある場合、 [`SubCmd::exec`](trait.SubCmd.html#tymethod.exec) が実行されるため制御は返らない。
    /// 制御が返った場合、一致するサブコマンドは存在しない。
    pub fn execute(&self, name: &str, matches: &ArgMatches, config: Config) {
        if let Some(cmd) = self.table.get(name) {
            cmd.exec(matches, config);
        }
    }

    /// サブコマンドを追加する。
    pub fn append(&mut self, subcmd: Box<dyn SubCmd>) {
        if let Some(exists) = self.table.insert(subcmd.name().to_owned(), subcmd) {
            panic!("registering duplecated subcommand: {}", exists.name());
        }
    }
}

impl fmt::Debug for SubCommandSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.table.keys()).finish()
    }
}
