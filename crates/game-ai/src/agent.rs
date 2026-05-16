mod entropy;
mod random;
mod search;
mod unfair;

pub use entropy::{entoropy, UseEntropyPlayer};
pub use random::RandomPlayer;
pub use search::{search_depth, search_rec, SearchPlayer};
pub use unfair::Unfair;

pub(crate) fn time_seed() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::{
        abstract_game::ImperfectInfoGame,
        config::default_config,
        defs::Move,
    };

    #[test]
    fn a() {
        let config = default_config();
        let mut game = config.gen_random(12345);
        game.move_game(Move::Query {
            query_to: 1,
            query_sort: "A".into(),
        });
        game.move_game(Move::Query {
            query_to: 2,
            query_sort: "A".into(),
        });
        game.move_game(Move::Query {
            query_to: 2,
            query_sort: "B".into(),
        });
        let info = game.info_and_move_now();
        let a = search_depth(&info.0, 4);
        eprintln!("{a:?}")
    }
}
