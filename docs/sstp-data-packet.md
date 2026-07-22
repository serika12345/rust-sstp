# SSTP Data Packet

## 仕様根拠

- Microsoft Open Specificationsの[MS-SSTP: Secure Socket Tunneling Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/c50ed240-56f3-4309-8e0c-1644898f0ea8)
- [MS-SSTP 2.2.3 Data Packet](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/1e71add9-7243-490a-b816-2174d5b3c179)
- 公開版: Protocol Revision 21.0、2024-04-23

Data Packetは、4バイトのSSTP通信ヘッダーと、それに続くpayloadから構成します。ヘッダーのCビットは0です。Lengthはヘッダーを含むパケット全長であり、payload長は`Length - 4`です。

MS-SSTP 2.2.3はpayloadをカプセル化した上位プロトコルのframeと定め、現在のプロトコルはPPPです。このPRではpayloadを所有権のある不透明なバイト列として扱い、PPPの内容は解釈しません。

理由: SSTPパケット境界の検証とPPP frameの解釈を分けると、`sstp-protocol`内で各層の失敗を独立して試験できるためです。

不採用理由: Data Packetの復号と同時にPPPを解釈すると、SSTPの長さ検証と上位プロトコルの責務が混在するためです。

## 完全な一個の入力

`SstpDataPacket::try_from`は、渡された入力全体が一個のData Packetである場合だけ成功します。

| 入力 | 結果 |
|---|---|
| 入力長がLengthと一致 | payloadを所有する`SstpDataPacket`を返す |
| 入力長がLengthより短い | `Truncated`として拒否 |
| 入力長がLengthより長い | `TrailingBytes`として拒否 |
| Cビットが1 | `ExpectedDataPacket`として拒否 |

理由: H1-2は一個の完全な入力のAPIであり、余剰バイトを無視すると呼び出し側の分割間違いを成功として扱うためです。

不採用理由: 先頭パケットだけを復号して残りを返すAPIは、複数パケットとTCPの断片化を扱うH1-3の責務であるため、このPRには含めません。

## 固定test vectorと境界値

次の値はMS-SSTP 2.2.1と2.2.3のフィールド定義から導出します。

| 入力または出力 | 意味 |
|---|---|
| `10 00 00 04` | payload長0、全長4の最小Data Packet |
| `10 00 00 08 ff 03 c0 21` | 4バイトのpayloadを持つ全長8のData Packet |
| `10 00 0f ff` + 4091バイトのpayload | 12ビットLengthで表現できる最大Data Packet |
| `10 00 00 08 ff 03 c0` | 宣言長に1バイト不足 |
| `10 00 00 04 00` | 宣言長の後ろに1バイト余剰 |

`SstpDataPayload`は0から4091バイトだけを構築可能にします。このドメイン型からData Packetを構築するため、符号化時のLengthとpayload長は常に一致します。

理由: 12ビットLengthの上限4095から4バイトのヘッダーを除いた値を、payloadの構築時に一度だけ検証するためです。

不採用理由: 符号化APIで生の`Vec<u8>`を受け取ると、長さ超過を実行時の各符号化経路で繰り返し扱うことになるためです。
