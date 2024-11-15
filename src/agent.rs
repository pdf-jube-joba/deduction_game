use super::{defs::*, utils::*};
use crate::abstract_game::{Agent, ImperfectInfoGame};
use rand::{rngs::ThreadRng, thread_rng};

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
                let Some(sort) = info
                    .config
                    .all_sort()
                    .into_iter()
                    .find(|s: &Sort| s.0 == sort)
                else {
                    println!("sort が正しく入力されなかった。");
                    continue;
                };
                return Move::Query {
                    query_to: info.config.player_turn(pl_to),
                    query_sort: sort,
                };
            } else if move_string == "A" {
                let mut sorts_of_cards = vec![];
                for _ in 0..info.config.head_num() {
                    input! {
                        n: usize,
                        sorts: [String; n],
                    }
                    sorts_of_cards.push(sorts);
                }
                let declare: Vec<_> = sorts_of_cards
                    .into_iter()
                    .map(|sort| sort.into_iter().map(Sort).collect())
                    .collect();
                return Move::Declare { declare };
            }
        }
    }
}

// 必ず当てれるときは当てるがそうじゃないときは可能な手からランダムに打つ。
#[derive(Debug, Clone)]
pub struct RandomPlayer {
    thread_rng: ThreadRng,
}

impl Default for RandomPlayer {
    fn default() -> Self {
        Self {
            thread_rng: thread_rng(),
        }
    }
}

impl Agent for RandomPlayer {
    type Game = Game;
    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        // answerable なとき
        if let Some(answer) = answerable(info.clone()) {
            return Move::Declare { declare: answer };
        }
        let possible_moves = query_at(info.clone());
        if possible_moves.is_empty() {
            let mut possible_declare = declare_at(info);
            possible_declare.next().unwrap() // possible declare がないのはありえないと思う。
        } else {
            random_vec(&mut self.thread_rng, possible_moves)
        }
    }
}

// 現在の履歴から可能な状態の全体を考え、各 query に対して可能な状態の回答の分布のエントロピーを計算する。
// 一番エントロピーが低いと、情報量がより得られているので、その手を選ぶ。
#[derive(Debug, Clone)]
pub struct UseEntropyPlayer;

impl Default for UseEntropyPlayer {
    fn default() -> Self {
        Self
    }
}

impl Agent for UseEntropyPlayer {
    type Game = Game;
    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        if let Some(answer) = answerable(info.clone()) {
            return Move::Declare { declare: answer };
        }
        let who = info.player_turn();
        let distrs = info.config.all_states();

        let (_, q) = query_at(info.clone())
            .into_iter()
            .map(|q| {
                let mut distribution = vec![0; info.config.cards_num()];
                for distr in &distrs {
                    let MoveAns::Query {
                        query_to: _,
                        query_sort: _,
                        ans,
                    } = answer(&info.config, distr, q.clone(), who)
                    else {
                        unreachable!()
                    };
                    distribution[ans] += 1;
                }
                let mut entropy: f64 = 0_f64;
                for i in distribution {
                    if i == 0 {
                        continue;
                    }
                    entropy += ((i as f64) / (distrs.len() as f64)) * (i as f64).log2();
                }
                (entropy, q)
            })
            .min_by(|(entropy1, _), (entropy2, _)| entropy1.partial_cmp(entropy2).unwrap())
            .unwrap();

        q
    }
}
