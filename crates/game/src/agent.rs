pub use game_ai::*;

use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    defs::*,
};
use std::collections::HashSet;

#[derive(Debug, Clone)]
#[cfg(target_arch = "x86_64")]
pub struct CUIUser;

#[cfg(target_arch = "x86_64")]
impl Default for CUIUser {
    fn default() -> Self {
        Self
    }
}

#[cfg(target_arch = "x86_64")]
impl Agent for CUIUser {
    type Game = Game;

    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        _possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        use proconio::input;

        println!("your turn, view: {:?}", info.view);
        loop {
            input! {
                move_string: String,
            }

            if move_string == "Q" {
                input! {
                    pl_to: usize,
                    sort: String,
                }
                let Some(sort) = info.config.all_sort().into_iter().find(|s: &Sort| *s == sort)
                else {
                    println!("sort が正しく入力されなかった。");
                    continue;
                };
                return Move::Query {
                    query_to: info.config.player_turn(pl_to),
                    query_sort: sort,
                };
            } else if move_string == "A" {
                let mut declare = HashSet::new();
                for _ in 0..info.config.head_num() {
                    input! {
                        n: usize,
                    }
                    declare.insert(n);
                }
                return Move::Declare {
                    declare: declare.into_iter().collect(),
                };
            }
        }
    }
}
