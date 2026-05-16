#![allow(unexpected_cfgs)]

use game_ai::{RandomPlayer, SearchPlayer, Unfair, UseEntropyPlayer};
use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    config::default_config,
    defs::{Game, Info, Move},
};
use rand::{rngs::SmallRng, SeedableRng};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebState {
    pub you: usize,
    pub current_turn: usize,
    pub your_turn: bool,
    pub winner: Option<Vec<usize>>,
    pub info: Info,
    pub possible_moves: Vec<Move>,
}

impl WebState {
    fn from_game(game: &Game, you: usize) -> Self {
        let winner = game.is_win();
        let current_turn = game.player_turn();
        let (base_info, current_moves) = game.info_and_move_now();
        let info = Info {
            config: base_info.config,
            query_answer: game.history(),
            view: game.view_from_player(you),
        };
        let possible_moves = if winner.is_none() && current_turn == you {
            current_moves
        } else {
            vec![]
        };
        Self {
            you,
            current_turn,
            your_turn: winner.is_none() && current_turn == you,
            winner,
            info,
            possible_moves,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WebAi {
    Random,
    Entropy,
    Search,
    Unfair,
}

#[wasm_bindgen]
pub struct WebGame {
    game: Game,
    ai_players: Vec<Option<Box<dyn Agent<Game = Game>>>>,
    user_player: usize,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: usize, user_player: usize, ai_json: &str) -> Result<WebGame, JsValue> {
        let config = default_config();
        if user_player >= config.player_num() {
            return Err(JsValue::from_str("user player is out of range"));
        }
        let ai = parse_ai(ai_json)?;
        if ai.len() + 1 != config.player_num() {
            return Err(JsValue::from_str("ai count must match player_num - 1"));
        }

        let game = config.gen_random(seed);
        let mut ai_players: Vec<Option<Box<dyn Agent<Game = Game>>>> =
            (0..config.player_num()).map(|_| None).collect();
        for (player, strategy) in (0..config.player_num())
            .filter(|player| *player != user_player)
            .zip(ai.into_iter())
        {
            ai_players[player] = Some(build_ai(strategy, seed, player));
        }

        let mut web_game = Self {
            game,
            ai_players,
            user_player,
        };
        web_game.run_ai_turns()?;
        Ok(web_game)
    }

    pub fn state_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&WebState::from_game(&self.game, self.user_player))
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    pub fn play_move_json(&mut self, move_json: &str) -> Result<String, JsValue> {
        if self.game.is_win().is_some() {
            return Err(JsValue::from_str("game is already over"));
        }
        if self.game.player_turn() != self.user_player {
            return Err(JsValue::from_str("it is not the user's turn"));
        }

        let action: Move = serde_json::from_str(move_json)
            .map_err(|err| JsValue::from_str(&format!("invalid move json: {err}")))?;
        if !self.game.move_game(action) {
            return Err(JsValue::from_str("illegal move"));
        }

        self.run_ai_turns()?;
        self.state_json()
    }
}

impl WebGame {
    fn run_ai_turns(&mut self) -> Result<(), JsValue> {
        while self.game.is_win().is_none() && self.game.player_turn() != self.user_player {
            let player = self.game.player_turn();
            let Some(agent) = self.ai_players[player].as_mut() else {
                return Err(JsValue::from_str("missing ai for non-user player"));
            };
            let (info, possible_moves) = self.game.info_and_move_now();
            let action = agent.use_info(info, possible_moves);
            if !self.game.move_game(action) {
                return Err(JsValue::from_str("ai produced an illegal move"));
            }
        }
        Ok(())
    }
}

fn parse_ai(ai_json: &str) -> Result<Vec<WebAi>, JsValue> {
    serde_json::from_str(ai_json)
        .map_err(|err| JsValue::from_str(&format!("invalid ai json: {err}")))
}

fn build_ai(strategy: WebAi, seed: usize, player: usize) -> Box<dyn Agent<Game = Game>> {
    match strategy {
        WebAi::Random => Box::new(RandomPlayer::new(SmallRng::seed_from_u64(
            seed as u64 + player as u64 + 1,
        ))),
        WebAi::Entropy => Box::new(UseEntropyPlayer),
        WebAi::Search => Box::new(SearchPlayer::new(2)),
        WebAi::Unfair => Box::new(Unfair::new(0.7)),
    }
}
