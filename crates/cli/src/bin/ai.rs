use cli::{get_json, post_json, JoinResponse, MoveRequest, StateResponse};
use game_ai::{RandomPlayer, SearchPlayer, Unfair, UseEntropyPlayer};
use game_core::{abstract_game::Agent, defs::Game};
use rand::rngs::ThreadRng;
use std::{thread, time::Duration};

fn main() {
    let strategy = std::env::args()
        .nth(1)
        .unwrap_or_else(|| usage_and_exit("ai <strategy> <port>"));
    let port = std::env::args()
        .nth(2)
        .unwrap_or_else(|| usage_and_exit("ai <strategy> <port>"))
        .parse::<u16>()
        .unwrap_or_else(|_| usage_and_exit("ai <strategy> <port>"));

    let joined: JoinResponse =
        post_json(port, "/join", &serde_json::json!({}), None).expect("failed to join game server");
    println!(
        "joined as player {} with strategy {}",
        joined.player, strategy
    );
    let secret = joined.secret;

    let mut agent = build_agent(&strategy);

    loop {
        let state: StateResponse =
            get_json(port, "/state", Some(&secret)).expect("failed to fetch state");

        if let Some(winner) = state.winner.clone() {
            println!("game finished: {winner:?}");
            break;
        }

        if !state.your_turn {
            thread::sleep(Duration::from_millis(1000));
            continue;
        }

        let info = state.info;
        let possible_moves = state.possible_moves;
        let action = agent.use_info(info, possible_moves);
        let _response: serde_json::Value =
            post_json(port, "/move", &MoveRequest { action }, Some(&secret))
                .expect("failed to submit move");
    }
}

fn build_agent(strategy: &str) -> Box<dyn Agent<Game = Game>> {
    match strategy {
        "random" => Box::new(RandomPlayer::<ThreadRng>::default()),
        "entropy" => Box::new(UseEntropyPlayer),
        "search" => Box::new(SearchPlayer::new(2)),
        "unfair" => Box::new(Unfair::new(0.7)),
        _ => usage_and_exit("ai <random|entropy|search|unfair> <port>"),
    }
}

fn usage_and_exit(message: &str) -> ! {
    eprintln!("usage: {message}");
    std::process::exit(2);
}
