sbak
=======

## 概要

`sbak` はファイルのハッシュ値ベースの簡易バックアップツールです。

## 使用法

### バックアップリポジトリの生成

まずバックアップ先となるリポジトリを作成します。

```sh
$ sbak init repo --path /backup/sbak_repository
```

リポジトリのパスを設定ファイルに記述しておくことで、以降の操作でリポジトリのパスを明示的に指定する必要がなくなります。

```toml
repository_path = "/backup/sbak_repository"
```

### Bankの生成

`sbak` では、1つのバックアップ対象ディレクトリに対して1つのBankを使用します。

```
$ sbak init bank --name sample_home_dir --path /home/sample
```

Bankにはバックアップの履歴や設定が保存されます。
ファイルの実体（オブジェクトファイル）は同じリポジトリの複数のBankで共有されます。


### バックアップの実行

`backup` サブコマンドでバックアップを実行できます。

```
$ sbak backup --bank sample_home_dir
```

Bank名を指定しなかった場合、全てのBankのバックアップが実行されます。

### 履歴一覧の表示

`history` サブコマンドで直近のバックアップ履歴の一覧を表示できます。

```
$ sbak history --bank sample_home_dir -n 5
2019-08-15 08:56:43    ba5685d1fc8703221e9c2dd1df3f9d1d5aa7d39b89f18f646479c1e287601224
2019-08-15 12:01:51    8137026f10033c85ffde22b790f63317cb2ed1cdf831d0ed1cc16230bf33a9d6
2019-08-15 12:16:46    3e0559f4b49eaf3c8aa442e9c740e35433f957f75381f836cc4ddbf4dba60115
2019-08-15 12:58:42    852ab268cde218d6d4e9fee1cb1573d61e15feeb7e688b687aa888a74afc940a
2019-08-15 14:20:59    7896920d3d8f38e9073960216d6638e41e04a73ebf1211d67db3225c8836fc6a
```

### ディレクトリの復元

`restore` サブコマンドで履歴からディレクトリを復元できます。
現状ではバックアップ対象ディレクトリを丸ごと復元します。

シンボリックリンクは保存されていますが、展開されません。
`--show-symlinks`オプションをつけることで、シンボリックリンクの一覧が出力されます。

```
$ sbak restore --bank sample_home_dir --revision 8137026f --to restored_dir
```


## 設定ファイル

```
repository_path = 'U:\sbak_repo'

[log]
output = 'stderr'
level = 'info'
```

### 全体設定

| 変数名 | 概要 | 有効な値 |
|:------|:-----|:--------|
| repository_path | 使用するリポジトリのパス |  |

### ログ設定

| 変数名 | 概要 | 有効な値 |
|:------|:-----|:--------|
| output | ログの出力先 | `stderr`、ログファイルのパス |
| level | ログレベル | `off`, `error`, `warn`, `info`, `debug`, `trace` |

## 除外設定ファイル

`.gitignore`と同様の`.sbakignore`ファイルをディレクトリ内に置くことで、指定したファイルをバックアップされないようにできます。

## License

`sbak` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).