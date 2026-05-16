use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    defs::Game,
    utils::{answerable_info, random_vec},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

// 必ず当てれるときは当てるがそうじゃないときは可能な手からランダムに打つ。
#[derive(Debug, Clone, PartialEq)]
pub struct RandomPlayer<R>
where
    R: Rng,
{
    rng: R,
}

impl<R> RandomPlayer<R>
where
    R: Rng,
{
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl Default for RandomPlayer<SmallRng> {
    fn default() -> Self {
        Self {
            rng: SmallRng::seed_from_u64(time_seed()),
        }
    }
}

impl<R> Agent for RandomPlayer<R>
where
    R: Rng,
{
    type Game = Game;

    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        _possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        if let Some(answer) = answerable_info(&info) {
            return answer;
        }
        let possible_moves = info.movable_query();
        if possible_moves.is_empty() {
            let possible_declare = info.movable_declare();
            possible_declare.into_iter().next().unwrap()
        } else {
            random_vec(&mut self.rng, possible_moves.into_iter().collect())
        }
    }
}

fn time_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}
