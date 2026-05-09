# deduction_game
次のような感じにしたい。
## crate の分け方
- game-core : 純粋なゲーム規則。State/Observation/Action/step
- game-protocol : JSON/MessagePack など。Observation, Action の入出力形式
- game-train : AI の学習。
- game-ai : AI の実装。Random/RuleBased/Policy など
- game-cli-server : CLI/stdio/TCP でゲームを進行するバイナリ
- game-cli-ai : stdio で Observation を受けて Action を返すバイナリ
- game-web : wasm。UI + game-core + game-ai を同じ wasm に入れてよい

## その他実装について
乱数については自分では生成せずにシードとして外部から与える。
