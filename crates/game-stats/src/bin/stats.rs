use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use game_stats::{default_config, run_match, Strategy};

fn main() {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(message) => usage_and_exit(&message),
    };

    let player_num = default_config().player_num();
    if args.strategies.len() != player_num {
        usage_and_exit(&format!(
            "expected {player_num} strategies for three_midium, got {}",
            args.strategies.len()
        ));
    }

    let config = default_config();
    let mut writer = output_writer(args.output.as_deref()).unwrap_or_else(|err| {
        eprintln!("failed to open output: {err}");
        std::process::exit(1);
    });

    for offset in 0..args.games {
        let seed = args.seed_start + offset;
        let record = run_match(&config, seed, &args.strategies);
        serde_json::to_writer(&mut writer, &record).expect("failed to serialize record");
        writer.write_all(b"\n").expect("failed to write newline");
    }
    writer.flush().expect("failed to flush output");
}

struct Args {
    games: usize,
    seed_start: usize,
    output: Option<String>,
    strategies: Vec<Strategy>,
}

fn parse_args(args: Vec<String>) -> Result<Args, String> {
    let mut games = 100;
    let mut seed_start = 0;
    let mut output = None;
    let mut strategies = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--games" => {
                i += 1;
                games = parse_usize(args.get(i), "--games")?;
            }
            "--seed-start" => {
                i += 1;
                seed_start = parse_usize(args.get(i), "--seed-start")?;
            }
            "--output" => {
                i += 1;
                output = Some(
                    args.get(i)
                        .ok_or_else(|| "missing value for --output".to_string())?
                        .clone(),
                );
            }
            "--strategies" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| "missing value for --strategies".to_string())?;
                strategies = Some(parse_strategies(raw)?);
            }
            flag => return Err(format!("unknown argument: {flag}")),
        }
        i += 1;
    }

    Ok(Args {
        games,
        seed_start,
        output,
        strategies: strategies.unwrap_or_else(default_strategies),
    })
}

fn parse_usize(value: Option<&String>, flag: &str) -> Result<usize, String> {
    value
        .ok_or_else(|| format!("missing value for {flag}"))?
        .parse::<usize>()
        .map_err(|_| format!("invalid integer for {flag}"))
}

fn parse_strategies(raw: &str) -> Result<Vec<Strategy>, String> {
    raw.split(',')
        .map(|name| Strategy::parse(name).ok_or_else(|| format!("unknown strategy: {name}")))
        .collect()
}

fn default_strategies() -> Vec<Strategy> {
    vec![Strategy::Random, Strategy::Entropy, Strategy::Unfair]
}

fn output_writer(path: Option<&str>) -> io::Result<Box<dyn Write>> {
    match path {
        Some(path) => Ok(Box::new(BufWriter::new(File::create(path)?))),
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
    }
}

fn usage_and_exit(message: &str) -> ! {
    eprintln!("{message}");
    eprintln!(
        "usage: stats [--games N] [--seed-start N] [--output PATH] [--strategies random,entropy,unfair]"
    );
    std::process::exit(2);
}
