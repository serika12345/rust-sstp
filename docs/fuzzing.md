# SSTP復号器のfuzzテスト

## 公開資料

- [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz): nightly Rust、対応実行環境、基本command
- [libFuzzer tutorial](https://github.com/google/fuzzing/blob/master/tutorial/libFuzzerTutorial.md): `-max_len`と`-max_total_time`を含む実行option
- [cargo-denyのlicense例外](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html): crateとversionを限定した例外設定

## 対象と性質

`fuzz/fuzz_targets/sstp_packet_decode.rs`は、同じ任意入力を次の三つの境界へ渡します。

- `SstpPacketHeader`の復号
- 一個の完全な`SstpDataPacket`の復号
- `SstpPacketStreamDecoder`による連続入力の復号

復号に成功したヘッダーとData Packetは再符号化し、再復号後に同じドメイン値となることを検査します。逐次decoderは消費長が入力範囲内であること、Data Packetとして完成したframeが同じ往復性を持つことを検査します。

理由: 成功と拒否の両方がある任意入力をcoverage-guided fuzzingで繰り返し、強制終了、範囲外参照、長さ検証の不整合を検出するためです。

不採用理由: 固定テストだけでは、バイト組合せと長さ宣言の広い入力空間を継続的に探索できないためです。

## 最小corpus

`fuzz/seeds/sstp_packet_decode`は、次のhex表現を保持します。`scripts/verify-fuzz`が実行ごとの一時ディレクトリへバイト列として展開します。

- 最小Data Packet
- PPP protocol値を含む8バイトData Packet
- 8バイトControl Packet
- 未対応Versionのヘッダー

理由: 有効なData/Control種別、正規化対象のReserved、失敗経路を最初のcoverageに含め、一時corpusに生成された入力をworktreeへ残さないためです。

不採用理由: 空corpusから開始すると、Version `0x10`と整合するLengthを同時に生成するまで成功経路の探索が遅れるためです。

## 実行時間の分離

`scripts/verify-fuzz`は通常検証向けに既定3秒、最大入力8192バイト、使用メモリ上限1024 MiBで実行します。`scripts/verify`はこの短時間走行を必須とします。

`scripts/verify-nightly`は、通常検証後に既定300秒の長時間走行を追加します。実行時間は次の環境変数で変更できます。

- `RUST_SSTP_FUZZ_SECONDS`: `scripts/verify-fuzz`の実行時間
- `RUST_SSTP_NIGHTLY_FUZZ_SECONDS`: `scripts/verify-nightly`の長時間fuzz実行時間

理由: PRごとの短時間検証を常に実行しつつ、探索時間の長いfuzzingを定期検証へ分離するためです。

不採用理由: 長時間fuzzingだけを用意するとPR検証から外れ、短時間走行だけでは探索の継続時間が不足するためです。

## toolchainと依存

- `cargo-fuzz 0.13.2`: Nixpkgsから取得するCLI。MIT OR Apache-2.0、rust-fuzz projectが保守
- `libfuzzer-sys 0.4.13`: `fuzz/Cargo.lock`で固定するLLVM libFuzzerのbinding。`(MIT OR Apache-2.0) AND NCSA`、rust-fuzz projectが保守
- `rust-overlay`: `flake.lock`で固定したfuzz専用nightly RustのNix入力

`cargo-fuzz`はsanitizerの不安定compiler optionを使うため、Nix wrapperのfuzz commandだけがnightly Rustを使います。通常のCargo、Clippy、テストはNixpkgsのstable Rustを維持します。

`libfuzzer-sys`のNCSAはこのfuzz workspaceの同一版にだけ`fuzz/deny.exceptions.toml`で許可します。root workspaceのApache-2.0の許可リストは変更しません。rootとfuzzの両方のlockfileに対し、RustSec、license、取得元、重複依存を検証します。

理由: cargo-fuzzとlibFuzzerはcoverage-guided fuzzing、sanitizer、crash inputの保存をRustの標準的なworkflowで提供し、版と依存をNixとCargoで固定できるためです。

不採用理由: property-basedテストだけではcoverage-guidedな入力探索とsanitizerを提供できません。AFLやHonggfuzzを並行導入すると、同じPRで複数のtoolchainとcorpus運用を抱えるため、現時点では採用しません。
