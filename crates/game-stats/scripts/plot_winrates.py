#!/usr/bin/env python3
import csv
import sys

import matplotlib.pyplot as plt


def main():
    if len(sys.argv) != 3:
        print("usage: plot_winrates.py <summary.csv> <output.png>", file=sys.stderr)
        sys.exit(2)

    names = []
    rates = []
    with open(sys.argv[1], "r", encoding="utf-8") as fh:
        reader = csv.DictReader(fh)
        for row in reader:
            names.append(row["strategy"])
            rates.append(float(row["win_rate"]))

    fig, ax = plt.subplots(figsize=(8, 4.5))
    ax.bar(names, rates, color="#33658a")
    ax.set_ylim(0, 1)
    ax.set_ylabel("win rate")
    ax.set_title("Win Rate by Strategy")
    fig.tight_layout()
    fig.savefig(sys.argv[2], dpi=160)


if __name__ == "__main__":
    main()
