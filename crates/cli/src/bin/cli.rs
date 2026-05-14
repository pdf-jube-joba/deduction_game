use cli::{get_json, post_json, JoinResponse, MoveRequest, StateResponse};
use game_core::defs::Move;
use std::{env, fs};

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        usage_and_exit();
    }

    let command = args.remove(0);
    match command.as_str() {
        "join" => command_join(args),
        "state" => command_state(args),
        "move" => command_move(args),
        _ => usage_and_exit(),
    }
}

fn command_join(args: Vec<String>) {
    let mut parser = ArgParser::new(args);
    let port = parser.port();
    let json = parser.take_flag("--json");
    parser.finish();

    let joined: JoinResponse =
        post_json(port, "/join", &serde_json::json!({}), None).expect("failed to join game server");

    if json {
        println!(
            "{}",
            serde_json::to_string(&joined).expect("failed to serialize join response")
        );
    } else {
        println!("player: {}", joined.player);
        println!("player_num: {}", joined.player_num);
        println!("secret: {}", joined.secret);
    }
}

fn command_state(args: Vec<String>) {
    let mut parser = ArgParser::new(args);
    let port = parser.port();
    let json = parser.take_flag("--json");
    let secret = parser.resolve_secret();
    parser.finish();

    let state: StateResponse =
        get_json(port, "/state", Some(&secret)).expect("failed to fetch state");

    if json {
        println!(
            "{}",
            serde_json::to_string(&state).expect("failed to serialize state response")
        );
    } else {
        println!("you: {}", state.you);
        println!("current_turn: {}", state.current_turn);
        println!("your_turn: {}", state.your_turn);
        println!("winner: {:?}", state.winner);
        println!("view: {:?}", state.info.view);
        println!("history:");
        for line in &state.history {
            println!("  {line}");
        }
        println!("possible_moves: {:?}", state.possible_moves);
    }
}

fn command_move(args: Vec<String>) {
    let mut parser = ArgParser::new(args);
    let port = parser.port();
    let json = parser.take_flag("--json");
    let secret = parser.resolve_secret();
    let action = parse_move(&mut parser);
    parser.finish();

    let response: serde_json::Value =
        post_json(port, "/move", &MoveRequest { action }, Some(&secret))
            .expect("failed to submit move");

    if json {
        println!(
            "{}",
            serde_json::to_string(&response).expect("failed to serialize move response")
        );
    } else {
        println!("{response}");
    }
}

fn parse_move(parser: &mut ArgParser) -> Move {
    let Some(kind) = parser.take_positional() else {
        usage_and_exit();
    };

    match kind.as_str() {
        "query" => {
            let query_to = parser
                .take_positional()
                .unwrap_or_else(|| usage_and_exit())
                .parse::<usize>()
                .unwrap_or_else(|_| usage_and_exit());
            let query_sort = parser.take_positional().unwrap_or_else(|| usage_and_exit());
            Move::Query {
                query_to,
                query_sort,
            }
        }
        "declare" => {
            let declare = parser
                .take_remaining_positionals()
                .into_iter()
                .map(|arg| arg.parse::<usize>().unwrap_or_else(|_| usage_and_exit()))
                .collect();
            Move::Declare { declare }
        }
        _ => usage_and_exit(),
    }
}

struct ArgParser {
    args: Vec<String>,
}

impl ArgParser {
    fn new(args: Vec<String>) -> Self {
        Self { args }
    }

    fn port(&mut self) -> u16 {
        self.take_positional()
            .unwrap_or_else(|| usage_and_exit())
            .parse::<u16>()
            .unwrap_or_else(|_| usage_and_exit())
    }

    fn take_flag(&mut self, flag: &str) -> bool {
        if let Some(index) = self.args.iter().position(|arg| arg == flag) {
            self.args.remove(index);
            true
        } else {
            false
        }
    }

    fn take_option(&mut self, flag: &str) -> Option<String> {
        let index = self.args.iter().position(|arg| arg == flag)?;
        self.args.remove(index);
        if index >= self.args.len() {
            usage_and_exit();
        }
        Some(self.args.remove(index))
    }

    fn take_positional(&mut self) -> Option<String> {
        let index = self.args.iter().position(|arg| !arg.starts_with("--"))?;
        Some(self.args.remove(index))
    }

    fn take_remaining_positionals(&mut self) -> Vec<String> {
        let mut positionals = Vec::new();
        while let Some(index) = self.args.iter().position(|arg| !arg.starts_with("--")) {
            positionals.push(self.args.remove(index));
        }
        positionals
    }

    fn resolve_secret(&mut self) -> String {
        let secret = self.take_option("--secret");
        let secret_file = self.take_option("--secret-file");

        if secret.is_some() && secret_file.is_some() {
            eprintln!("--secret and --secret-file cannot be used together");
            std::process::exit(2);
        }

        if let Some(secret) = secret {
            return secret;
        }
        if let Some(path) = secret_file {
            return fs::read_to_string(path)
                .expect("failed to read secret file")
                .trim()
                .to_string();
        }
        if let Ok(secret) = env::var("GAME_SECRET") {
            if !secret.trim().is_empty() {
                return secret;
            }
        }

        eprintln!("missing secret: use --secret, --secret-file, or GAME_SECRET");
        std::process::exit(2);
    }

    fn finish(self) {
        if !self.args.is_empty() {
            usage_and_exit();
        }
    }
}

fn usage_and_exit() -> ! {
    eprintln!("usage:");
    eprintln!("  cli join <port> [--json]");
    eprintln!("  cli state <port> [--secret <secret> | --secret-file <path>] [--json]");
    eprintln!("  cli move <port> [--secret <secret> | --secret-file <path>] [--json] query <player> <sort>");
    eprintln!(
        "  cli move <port> [--secret <secret> | --secret-file <path>] [--json] declare <card>..."
    );
    std::process::exit(2);
}
