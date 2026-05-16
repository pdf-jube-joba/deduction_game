use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use game_stats::{
    run_match, stats_config, summarize_records, write_summary_data, MatchRecord, Strategy,
};

fn main() {
    let args = match parse_args(std::env::args().skip(1).collect()) {
        Ok(args) => args,
        Err(message) => usage_and_exit(&message),
    };

    let config = stats_config();
    let mut records_writer = output_writer(args.output.as_deref()).unwrap_or_else(|err| {
        eprintln!("failed to open records output: {err}");
        std::process::exit(1);
    });
    let mut records = Vec::with_capacity(args.games);

    for seed in 0..args.games {
        let record = run_match(&config, seed, &args.strategies);
        serde_json::to_writer(&mut records_writer, &record).expect("failed to serialize record");
        records_writer
            .write_all(b"\n")
            .expect("failed to write newline");
        records.push(record);
    }
    records_writer.flush().expect("failed to flush records output");

    if let Some(path) = args.summary_out.as_deref() {
        write_summary_file(path, &records).unwrap_or_else(|err| {
            eprintln!("failed to write summary output: {err}");
            std::process::exit(1);
        });
    }
}

struct Args {
    games: usize,
    output: Option<String>,
    summary_out: Option<String>,
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
    let mut output = None;
    let mut summary_out = None;

    let mut i = player_num;
    while i < args.len() {
        match args[i].as_str() {
            "--games" => {
                i += 1;
                games = parse_usize(args.get(i), "--games")?;
            }
            "--output" => {
                i += 1;
                output = Some(
                    args.get(i)
                        .ok_or_else(|| "missing value for --output".to_string())?
                        .clone(),
                );
            }
            "--summary-out" => {
                i += 1;
                summary_out = Some(
                    args.get(i)
                        .ok_or_else(|| "missing value for --summary-out".to_string())?
                        .clone(),
                );
            }
            flag => return Err(format!("unknown argument: {flag}")),
        }
        i += 1;
    }

    Ok(Args {
        games,
        output,
        summary_out,
        strategies,
    })
}

fn parse_usize(value: Option<&String>, flag: &str) -> Result<usize, String> {
    value
        .ok_or_else(|| format!("missing value for {flag}"))?
        .parse::<usize>()
        .map_err(|_| format!("invalid integer for {flag}"))
}

fn output_writer(path: Option<&str>) -> io::Result<Box<dyn Write>> {
    match path {
        Some(path) => Ok(Box::new(BufWriter::new(File::create(path)?))),
        None => Ok(Box::new(BufWriter::new(io::stdout()))),
    }
}

fn write_summary_file(path: &str, records: &[MatchRecord]) -> io::Result<()> {
    let data = write_summary_data(&summarize_records(records));
    let mut writer = BufWriter::new(File::create(path)?);
    writer.write_all(data.as_bytes())?;
    writer.flush()
}

fn usage_and_exit(message: &str) -> ! {
    eprintln!("{message}");
    eprintln!(
        "usage: game-stats <p0> <p1> <p2> [--games N] [--output PATH] [--summary-out PATH]"
    );
    eprintln!("example: cargo run -p game-stats -- random entropy unfair --games 200");
    std::process::exit(2);
}
