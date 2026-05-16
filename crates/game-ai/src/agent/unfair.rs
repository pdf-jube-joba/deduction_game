use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    defs::*,
    utils::*,
};

use super::entoropy;

#[derive(Debug, Clone, PartialEq)]
pub struct Unfair {
    first: bool,
    attack: f64,
}

impl Unfair {
    pub fn new(attack: f64) -> Self {
        Unfair {
            first: true,
            attack,
        }
    }
}

impl Agent for Unfair {
    type Game = Game;

    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        _possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        if let Some(answer) = answerable(&info.config, &info.query_answer, &info.view) {
            return answer;
        }
        if self.first {
            self.first = false;
            return entoropy(info).unwrap();
        }

        let head_numed = possible_head_numed(&info.config, &info.query_answer, &info.view);
        assert!(!head_numed.is_empty());
        let mut num_all = 0;
        let (a, num) = head_numed
            .into_iter()
            .map(|(declare, num)| {
                num_all += num;
                let m = Move::Declare { declare };
                (m, num)
            })
            .max_by_key(|(_, n)| *n)
            .unwrap();

        if (num as f64 / num_all as f64) >= self.attack {
            a
        } else {
            entoropy(info).unwrap()
        }
    }
}
