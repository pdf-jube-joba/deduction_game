# game-stats scripts

Rust 側で JSONL と `data.txt` を吐いて、ここでは gnuplot 用の設定だけを持つ。

例:

```bash
cargo run -p game-stats -- random entropy unfair --games 200 --output records.jsonl --summary-out data.txt
gnuplot -c crates/game-stats/scripts/winrates.plt
```

`data.txt` は空白区切りで、列は次の順:

```text
# strategy games wins win_rate avg_moves avg_think_ms
```

`search` は `three_midium` だと重いので、使う場合は `cargo run -p game-stats -- random entropy search --games 10 ...` のように少なめで回す。
