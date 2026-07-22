# rust-sstp

高速・安全・クロスプラットフォームなSSTPクライアントをRustで実装するためのプロジェクトです。通信中核は利用者画面やオペレーティングシステム接続仕様から独立させ、Linux、Android、macOS、iOSのVPNインターフェースへ薄い接続部で接続できる構成を目指します。

## スコープ

- 実装するもの: SSTPクライアント、純粋なプロトコル層、接続状態機械、各対象オペレーティングシステムの接続部、将来のFlutter利用者画面との外部関数境界
- 対象オペレーティングシステム: Linux、Android、macOS、iOS
- WindowsのネイティブクライアントとWindows Server RRASは、相互運用性・性能比較の参照実装として利用します。

詳細は [DESIGN_PLAN.md](DESIGN_PLAN.md) と [docs/scope.md](docs/scope.md) を参照してください。

## 現在の構成

- `sstp-protocol`: SSTPの語彙と通信形式。I/Oを持たない純粋なドメイン層。
- `sstp-session`: 接続ライフサイクルを表す純粋な状態機械。
- `sstp-harness`: 開発中のプロトコル動作を観察するコマンド行インターフェース兼I/O境界。

オペレーティングシステム接続部、TLS、HTTP、PPP、Flutter連携は、対応する仕様と試験を確定してから追加します。先に汎用抽象を作りません。

## 開発環境

Nix Flakeを唯一の正規開発環境とし、direnvから読み込みます。

```sh
direnv allow
nix develop
./scripts/verify
```

direnvを使わない場合も、コマンドはNix環境内で実行します。

```sh
nix develop -c cargo test --workspace
```

開発用試験基盤は次のように実行できます。

```sh
nix develop -c cargo run -p sstp-harness -- plan-connect
```

## 作業規律

- 実装順序とPR単位の完了条件は [TODO.md](TODO.md) を唯一の正として扱います。各チェック項目を単独でレビュー・統合できる一つのPRとして、上から順に進めます。
- 未確定の設計判断は [docs/design-todo.md](docs/design-todo.md) に置きます。
- 実施結果と検証記録は [docs/progress.md](docs/progress.md) に追記します。
- コーディング規約は [docs/coding-rules.md](docs/coding-rules.md) に従います。
- 再現可能な作業手順は `.agents/skills/rust-sstp-*` にあります。

## ライセンス

Apache License 2.0。依存関係の方針は [docs/supply-chain.md](docs/supply-chain.md) を参照してください。
