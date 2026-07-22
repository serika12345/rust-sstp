---
name: rust-sstp-advance-small
description: 現在のPR内で、テスト駆動による最小の増分を実装する。
---

# 一つの小さな増分を進める

1. `AGENTS.md`と`TODO.md`の進行中部分を読む。
2. PRを広げない未完了の振る舞いを一つ選ぶ。
3. 先に失敗する試験を追加し、意図した原因で失敗することを確認する。
4. ドメイン固有の最小変更を実装し、`sstp-harness -> sstp-session -> sstp-protocol`の依存方向を保つ。
5. 対象を絞った試験を行い、その後 `nix develop -c ./scripts/verify-quick` を実行する。
6. 検証済みの作業だけを `TODO.md` へ反映し、正確な証跡を `docs/progress.md` へ追記する。
7. 新しい公開抽象、依存、`unsafe`範囲、未解決の設計判断が必要なら停止し、`docs/design-todo.md`へ記録する。
