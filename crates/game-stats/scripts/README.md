# game-stats scripts

Rust 側で JSONL を吐いて、ここで集計やグラフ化をする。

例:

```bash
cargo run -p game-stats --bin stats -- --games 200 --output records.jsonl
python3 crates/game-stats/scripts/summarize.py records.jsonl > summary.csv
python3 crates/game-stats/scripts/plot_winrates.py summary.csv winrates.png
```

`search` は `three_midium` だと重いので、使う場合は `--games` を小さめにして `--strategies random,entropy,search` のように明示指定する。
