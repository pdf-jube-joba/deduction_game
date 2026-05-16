# deduction_game
次のような感じにしたい。
## crate の分け方
- game-core : 純粋なゲーム規則。State/Info/Action/step
  - Serialize をつける。
- game-train : AI の学習。
  - バッチで動かす？
- game-ai/random / game-ai/entropy / game-ai/search / game-ai/unfair : AI の実装。
  - 学習や事前計算の結果は各 AI ごとに持つ。
- game-cli : `bin/` 以下に3つ用意する
  - game-cli-server : HTTP でゲームを進行する（シードで生成、状態はメモリで保管する）。
  - game-cli-client : one-shot のコマンドにして HTTP 通信、ゲームを進める。
  - game-cli-ai : 各 AI crate を使って、 HTTP で localhost と通信して行動する。
    - polling (loop 内で sleep して毎回聞きに行く)
- game-web : wasm。game-core + 各 AI crate を同じ wasm に入れてよい

## その他実装について
- 乱数については自分では生成せずにシードとして外部から与える。
- CLI で遊ぶ場合に、 codex がゲーム状態を閲覧できないようにする必要がある
  - HTTP で API を開いて対戦できるようにする。
  - ゲーム状態はメモリに置き、シードは外部から random で与えて見えないようにする。

## web について
yew やら web-sys やらを無理に使う必要はなくて、
普通に wasm-bindgen で html/css/js と通信して ui は html/css/js で書く方がいい。
