# game-stats scripts

Rust 側で JSONL を吐いて、ここでは集計や gnuplot 用の設定を持つ。

例:

```bash
cargo run -p game-stats -- random entropy unfair --games 200 \
  | tee records.jsonl \
  | python3 crates/game-stats/scripts/summarize.py \
  > data.txt
gnuplot -c crates/game-stats/scripts/winrates.plt
```

`data.txt` は空白区切りで、列は次の順:

```text
# strategy games wins win_rate avg_moves avg_think_ms
```

`records.jsonl` は `tee` で保存したときだけ残る。各試合の `history` も入るので、勝った試合の進行をあとから直接確認できる。

`search` は `three_midium` だと重いので、使う場合は `cargo run -p game-stats -- random entropy search --games 10 ...` のように少なめで回す。
