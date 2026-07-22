# SSTP通信ヘッダー

## 仕様根拠

- Microsoft Open Specificationsの[MS-SSTP: Secure Socket Tunneling Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/c50ed240-56f3-4309-8e0c-1644898f0ea8)
- [MS-SSTP 2.2.1 SSTP Packet](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/2991892f-fefc-4129-adac-cd6a5d04bb48)
- 公開版: Protocol Revision 21.0、2024-04-23

通信ヘッダーは4バイトです。各フィールドをネットワークバイトオーダーで次のように解釈します。

| バイト | フィールド | 検証 |
|---|---|---|
| 0 | Version | SSTP 1.0を表す`0x10`だけを受理する |
| 1 | 上位7ビットReserved、下位1ビットC | Cが0ならData、1ならControl。Reservedは受信時に無視する |
| 2〜3 | 上位4ビットR、下位12ビットLength | Rは受信時に無視する。Lengthはヘッダーを含むパケット全長 |

12ビットのLengthから、表現可能な最大値は4095です。Lengthは4バイトのヘッダー自体を含むため、この実装が受理する最小値は4です。

## 固定test vector

次の値はMS-SSTP 2.2.1のフィールド定義から導出します。

| 入力または出力 | 意味 |
|---|---|
| `10 00 00 04` | SSTP 1.0、Data、全長4 |
| `10 01 01 23` | SSTP 1.0、Control、全長291 |
| `10 00 0f ff` | SSTP 1.0、Data、最大長4095 |
| `10 fe f0 04` | Reservedが非0でも、Data、全長4として復号する |
| `10 ff f0 08` | Reservedを無視してControl、全長8として復号し、再符号化結果を`10 01 00 08`に正規化する |
| `11 00 00 04` | 未対応Versionとして拒否する |
| `10 00 00 03` | ヘッダーより短いLengthとして拒否する |

## Reservedの扱い

送信時はReservedとRを0にし、受信時は値を無視します。復号したヘッダーにはReservedを保持せず、再符号化時に0へ正規化します。

理由: MS-SSTP 2.2.1がReservedを送信時0、受信時無視と定めており、将来の拡張ビットを持つ受信パケットも現在理解できるフィールドだけで処理できるためです。

不採用理由: Reservedが非0の受信パケットを拒否すると、受信時に無視するという仕様に反し、将来の拡張との互換性を失うためです。

## Lengthのドメイン型

`SstpPacketLength`は4から4095だけを構築可能にします。通信ヘッダーの公開APIは`SstpPacketLength`を使い、生の`u16`へパケット長の意味と検証を委ねません。

理由: ヘッダー未満の長さと12ビットを超える長さを、符号化・復号より前に型の境界で排除するためです。

不採用理由: 生の`u16`を公開APIで受け取ると、範囲検証が呼び出し側ごとに分散し、上位4ビットのReservedとの区別も型から読み取れないためです。
