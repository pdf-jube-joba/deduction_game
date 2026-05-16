use std::io::{self, BufWriter, Write};

use game_stats::{run_match, stats_config, Strategy};

fn main() {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(message) => usage_and_exit(&message),
    };

    let config = stats_config();
    let stdout = io::stdout();
    let mut records_writer = BufWriter::new(stdout.lock());

    for seed in 0..args.games {
        let record = run_match(&config, seed, &args.strategies);
        serde_json::to_writer(&mut records_writer, &record).expect("failed to serialize record");
        records_writer
            .write_all(b"\n")
            .expect("failed to write newline");
    }
    records_writer.flush().expect("failed to flush records output");
}

struct Args {
    games: usize,
    strategies: Vec<Strategy>,
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
    let player_num = stats_config().player_num();
    if args.len() < player_num {
        return Err(format!(
            "expected {player_num} positional strategies like: random entropy unfair"
        ));
    }

    let strategies = args[..player_num]
        .iter()
        .map(|name| Strategy::parse(name).ok_or_else(|| format!("unknown strategy: {name}")))
        .collect::<Result<Vec<_>, _>>()?;

    let mut games = 100;

    let mut i = player_num;
    while i < args.len() {
        match args[i].as_str() {
            "--games" => {
                i += 1;
                games = parse_usize(args.get(i), "--games")?;
            }
            flag => return Err(format!("unknown argument: {flag}")),
        }
        i += 1;
    }

    Ok(Args {
        games,
        strategies,
    })
}

fn parse_usize(value: Option<&String>, flag: &str) -> Result<usize, String> {
    value
        .ok_or_else(|| format!("missing value for {flag}"))?
        .parse::<usize>()
        .map_err(|_| format!("invalid integer for {flag}"))
}

fn usage_and_exit(message: &str) -> ! {
    eprintln!("{message}");
    eprintln!("usage: game-stats <p0> <p1> <p2> [--games N]");
    eprintln!("example: cargo run -p game-stats -- random entropy unfair --games 200");
    std::process::exit(2);
}
