use std::time::Instant;

use game_ai_entropy::UseEntropyPlayer;
use game_ai_random::RandomPlayer;
use game_ai_search::SearchPlayer;
use game_ai_unfair::Unfair;
use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    config::three_midium,
    defs::{Game, GameConfig, Move, MoveAns},
};
use rand::{rngs::SmallRng, SeedableRng};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Random,
    Entropy,
    Search,
    Unfair,
}

impl Strategy {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "random" => Some(Self::Random),
            "entropy" => Some(Self::Entropy),
            "search" => Some(Self::Search),
            "unfair" => Some(Self::Unfair),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::Entropy => "entropy",
            Self::Search => "search",
            Self::Unfair => "unfair",
        }
    }
}

struct AgentSlot {
    name: &'static str,
    agent: Box<dyn Agent<Game = Game>>,
    stats: AgentStats,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct AgentStats {
    pub move_count: usize,
    pub query_count: usize,
    pub declare_count: usize,
    pub think_nanos: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchRecord {
    pub config: &'static str,
    pub seed: usize,
    pub players: Vec<&'static str>,
    pub winner: Vec<usize>,
    pub turns: usize,
    pub elapsed_nanos: u128,
    pub agents: Vec<AgentStats>,
    pub history: Vec<MoveAns>,
}

pub fn stats_config_name() -> &'static str {
    "three_midium"
}

pub fn stats_config() -> GameConfig {
    three_midium()
}

pub fn run_match(config: &GameConfig, seed: usize, strategies: &[Strategy]) -> MatchRecord {
    let mut game = config.gen_random(seed);
    let mut slots: Vec<_> = strategies
        .iter()
        .enumerate()
        .map(|(player, strategy)| AgentSlot {
            name: strategy.as_str(),
            agent: build_agent(*strategy, seed, player),
            stats: AgentStats::default(),
        })
        .collect();

    let started = Instant::now();
    let mut turns = 0;
    while game.is_win().is_none() {
        let player = game.player_turn();
        let (info, possible_moves) = game.info_and_move_now();
        let slot = &mut slots[player];
        let think_started = Instant::now();
        let action = slot.agent.use_info(info, possible_moves);
        let think_elapsed = think_started.elapsed().as_nanos();
        slot.stats.move_count += 1;
        slot.stats.think_nanos += think_elapsed;
        match &action {
            Move::Query { .. } => slot.stats.query_count += 1,
            Move::Declare { .. } => slot.stats.declare_count += 1,
        }
        if !game.move_game(action) {
            panic!("agent produced illegal move");
        }
        turns += 1;
    }

    MatchRecord {
        config: stats_config_name(),
        seed,
        players: slots.iter().map(|slot| slot.name).collect(),
        winner: game.is_win().unwrap(),
        turns,
        elapsed_nanos: started.elapsed().as_nanos(),
        agents: slots.into_iter().map(|slot| slot.stats).collect(),
        history: game.history(),
    }
}

fn build_agent(strategy: Strategy, seed: usize, player: usize) -> Box<dyn Agent<Game = Game>> {
    match strategy {
        Strategy::Random => Box::new(RandomPlayer::new(SmallRng::seed_from_u64(
            seed as u64 + player as u64 + 1,
        ))),
        Strategy::Entropy => Box::new(UseEntropyPlayer),
        Strategy::Search => Box::new(SearchPlayer::new(2)),
        Strategy::Unfair => Box::new(Unfair::new(0.7)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_match_returns_stats_for_all_players() {
        let config = stats_config();
        let record = run_match(
            &config,
            42,
            &[Strategy::Random, Strategy::Entropy, Strategy::Search],
        );
        assert_eq!(record.config, "three_midium");
        assert_eq!(record.players.len(), 3);
        assert_eq!(record.agents.len(), 3);
        assert_eq!(record.winner.len(), 3);
        assert!(record.turns > 0);
        assert_eq!(record.history.len(), record.turns);
    }
}
