# game-stats
- 統計を取る：`cargo run -p game-stats -- random entropy unfair --games 200 --output records.jsonl --summary-out data.txt`
  - 既定の config は `three_midium`。
  - 戦略は位置引数 3 つで、その順に Player 0, 1, 2 へ入る。
- 結果：`data.txt` は `# strategy games wins win_rate avg_moves avg_think_ms` の列で出る。
- グラフ化： `gnuplot -c crates/game-stats/scripts/winrates.plt` を使う。
