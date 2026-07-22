# SSTPパケット逐次復号器

## 仕様根拠

- Microsoft Open Specificationsの[MS-SSTP: Secure Socket Tunneling Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/c50ed240-56f3-4309-8e0c-1644898f0ea8)
- [MS-SSTP 2.1 Transport](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/c9de1390-4d8f-4715-a257-221400b06d74)
- [MS-SSTP 2.2.1 SSTP Packet](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/2991892f-fefc-4129-adac-cd6a5d04bb48)
- [MS-SSTP 2.2.3 Data Packet](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-sstp/1e71add9-7243-490a-b816-2174d5b3c179)
- 公開版: Protocol Revision 21.0、2024-04-23

SSTPはHTTPSによるメッセージの信頼性のある配送を利用します。HTTPSを運ぶTCP入力の境界と、Lengthで定まるSSTPパケット境界が同じであるとは限りません。そのため、復号器は一個未満の断片、一個のパケット、複数パケットを含む結合入力を同じAPIで扱います。

理由: TCPまたはTLS recordの境界に依存せず、SSTPヘッダーのLengthだけをパケット境界の根拠にするためです。

不採用理由: 一回のreadに一個のSSTPパケットが届くとみなすと、TCPまたはTLSによる断片化と結合で復号できないためです。

## APIの契約

`SstpPacketStreamDecoder::decode_next`は、渡された入力から一個のSSTPパケットを完成させるために必要な分だけを消費します。結果とエラーは`SstpStreamConsumedBytes`を返すため、呼び出し側は同じ入力に残る次のパケットを続けて渡せます。

| 状態 | 結果 |
|---|---|
| ヘッダーまたは宣言長までバイトが足りない | `Incomplete`と消費済み入力長 |
| 一個のDataまたはControl Packetが完成 | `SstpPacketFrame`を持つ`Packet`と消費済み入力長 |
| 未対応Versionまたは不正長 | `SstpPacketStreamDecodeError`とエラー検出までの消費済み入力長 |

`Incomplete`は入力断片の正常な待機状態であり、不正入力のエラーとは区別します。エラー後はパケット境界を信頼できないため、その復号器で処理を続けません。

`SstpPacketFrame`は検証済みヘッダーと宣言長に一致するバイトを所有します。Data Packetは`SstpDataPacket::try_from`で意味解析します。Control Packetは境界と種別だけを保持し、メッセージと属性の意味解析はH2で追加します。

理由: 入力の消費量をドメイン型で明示すると、結合入力の未消費部分を重複または取りこぼしなく処理できるためです。

不採用理由: 入力全体を復号器が常に所有するAPIは、複数パケットを含む大きな入力を内部bufferへ不要に複製するためです。

## buffer上限

復号器は次の一個のSSTPパケットに必要なバイトだけを内部bufferに保持します。Lengthは12ビットのため、保持バイト数は4095以下です。結合入力の次パケット以降は複製せず、消費済み入力長で呼び出し側へ残します。

理由: 入力chunkの大きさに関係なく、復号器が保持する外部入力をSSTPの最大パケット長までに限定するためです。

不採用理由: 宣言長を検査せず入力chunk全体を蓄積すると、上限のないメモリ使用を外部入力から引き起こせるためです。

## 試験範囲

2個のData Packetを連結した12バイトの入力に対し、11箇所の境界を分割する全2048通りを決定論的に生成し、すべて同じ二個のパケットに復号されることを確認します。また、空入力、ヘッダー断片、結合入力の消費長、不正Version、Control Packetの境界保持、最大4095バイトの1バイトごとの入力も検査します。

理由: 特定の分割例だけではなく、ヘッダーとpayloadのすべての境界に同じ復号規則が適用されることを確認するためです。

不採用理由: ヘッダーとpayloadの代表的な分割例だけを個別に書くと、未試験の分割境界が残るためです。
