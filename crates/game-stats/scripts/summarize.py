#!/usr/bin/env python3
import json
import sys
from collections import Counter, defaultdict


def load_records():
    for line in sys.stdin:
        line = line.strip()
        if line:
            yield json.loads(line)


def main():
    if len(sys.argv) != 1:
        print("usage: summarize.py < records.jsonl", file=sys.stderr)
        sys.exit(2)

    wins = Counter()
    games = Counter()
    turns = defaultdict(int)
    think_nanos = defaultdict(int)

    for record in load_records():
        for idx, name in enumerate(record["players"]):
            games[name] += 1
            turns[name] += record["agents"][idx]["move_count"]
            think_nanos[name] += record["agents"][idx]["think_nanos"]
            if record["winner"][idx]:
                wins[name] += 1

    print("# strategy games wins win_rate avg_moves avg_think_ms")
    for name in sorted(games):
        game_count = games[name]
        avg_moves = turns[name] / game_count if game_count else 0.0
        avg_think_ms = think_nanos[name] / game_count / 1_000_000 if game_count else 0.0
        print(
            f"{name} {game_count} {wins[name]} {wins[name] / game_count:.4f} {avg_moves:.2f} {avg_think_ms:.3f}"
        )


if __name__ == "__main__":
    main()
