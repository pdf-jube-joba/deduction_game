# game-stats
- 試合を組む： `cargo run -p game-stats -- random entropy unfair --games 200` stdout に出力する
  - 既定の config は `three_midium`。
  - 戦略は位置引数 3 つで、その順に Player 0, 1, 2 へ入る。
- 統計を取る：`python3 crates/game-stats/scripts/summarize.py` stdin 経由でえた試合経過をもとに統計を stdout に出力する
  - 結果：`# strategy games wins win_rate avg_moves avg_think_ms` の列で出る。
- グラフ化： `gnuplot -c crates/game-stats/scripts/winrates.plt` を使う。
