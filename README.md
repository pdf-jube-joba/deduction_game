# deduction_game
次のような感じにしたい。
## crate の分け方
- game-core : 純粋なゲーム規則。State/Observation/Action/step
- game-protocol : Observation, Action の入出力形式
- game-train : AI の学習。
  - バッチで動かす？
- game-ai : AI の実装。Random/RuleBased/Policy など
- game-cli-server : CLI/stdio/TCP でゲームを進行するバイナリ
- game-cli-ai : stdio で Observation を受けて Action を返すバイナリ
- game-web : wasm。UI + game-core + game-ai を同じ wasm に入れてよい

## その他実装について
- 乱数については自分では生成せずにシードとして外部から与える。
- CLI で遊ぶ場合に、 codex がゲーム状態を閲覧できないようにする必要がある
  - HTTP で API を開いて対戦できるようにする。
  - ゲーム状態はメモリに置き、シードは外部から random で与えて見えないようにする。

## web について
yew やら web-sys やらを無理に使う必要はなくて、
普通に wasm-bindgen で html/css/js と通信して ui は html/css/js で書く方がいい。
