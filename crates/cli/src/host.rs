use crate::{JoinResponse, MoveResponse, StateResponse};
use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    defs::{Game, GameConfig, Info, Move},
};
use rand::random;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct GameHost {
    game: Game,
    joined: Vec<Option<String>>,
}

impl GameHost {
    pub fn new(config: GameConfig) -> Self {
        let game = config.gen_random(random());
        let joined = vec![None; game.player_number()];
        Self { game, joined }
    }

    pub fn player_num(&self) -> usize {
        self.game.player_number()
    }

    pub fn join(&mut self) -> Option<JoinResponse> {
        let player = self.joined.iter().position(|joined| joined.is_none())?;
        let secret = self.generate_secret();
        self.joined[player] = Some(secret.clone());
        Some(JoinResponse {
            player,
            player_num: self.player_num(),
            secret,
        })
    }

    pub fn state_for_secret(&self, secret: &str) -> Option<StateResponse> {
        let player = self.player_from_secret(secret)?;

        let winner = self.game.is_win();
        let current_turn = self.game.player_turn();
        let (base_info, possible_moves) = self.game.info_and_move_now();
        let info = Info {
            config: base_info.config,
            query_answer: self.game.history(),
            view: self.game.view_from_player(player),
        };
        let possible_moves = if current_turn == player && winner.is_none() {
            possible_moves
        } else {
            vec![]
        };
        Some(StateResponse::from_info(
            player,
            current_turn,
            winner,
            info,
            possible_moves,
        ))
    }

    pub fn apply_move(&mut self, secret: &str, action: Move) -> Result<MoveResponse, String> {
        let player = self
            .player_from_secret(secret)
            .ok_or_else(|| "invalid secret".to_string())?;
        if self.game.is_win().is_some() {
            return Err("game is already over".into());
        }
        if self.game.player_turn() != player {
            return Err(format!("not player {player}'s turn"));
        }

        if !self.game.move_game(action) {
            return Err("illegal move".into());
        }

        Ok(MoveResponse {
            accepted: true,
            winner: self.game.is_win(),
        })
    }

    pub fn run_turn(
        &mut self,
        secret: &str,
        agent: &mut dyn Agent<Game = Game>,
    ) -> Result<MoveResponse, String> {
        let player = self
            .player_from_secret(secret)
            .ok_or_else(|| "invalid secret".to_string())?;
        if self.game.player_turn() != player {
            return Err(format!("not player {player}'s turn"));
        }
        let (info, possible_moves) = self.game.info_and_move_now();
        let action = agent.use_info(info, possible_moves);
        self.apply_move(secret, action)
    }

    fn player_from_secret(&self, secret: &str) -> Option<usize> {
        self.joined
            .iter()
            .position(|joined| joined.as_deref() == Some(secret))
    }

    fn generate_secret(&self) -> String {
        loop {
            let candidate = format!("{:032x}{:032x}", random::<u128>(), random::<u128>());
            if self.player_from_secret(&candidate).is_none() {
                return candidate;
            }
        }
    }
}

pub type SharedHost = Arc<Mutex<GameHost>>;
