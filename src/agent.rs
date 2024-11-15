
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::collections::HashSet;

use super::{game::*, utils::*};
pub trait Agent {
    fn which(
        &mut self,
        config: GameConfig,
        player: Player,
        view: View,
        history: Vec<(Move, Option<Ans>)>,
    ) -> Move;
    fn game(&mut self, game: &mut Game) -> Option<Ans> {
        let p = game.turn();
        let m = self.which(
            game.config().clone(),
            p,
            game.view_from_player(p),
            game.history(),
        );
        game.move_game(m)
    }
}

#[derive(Debug, Clone)]
#[cfg(target_arch = "x86_64")]
pub struct User;

#[cfg(target_arch = "x86_64")]
impl Default for User {
    fn default() -> Self {
        Self
    }
}

#[cfg(target_arch = "x86_64")]
impl Agent for User {
    fn which(
        &mut self,
        config: GameConfig,
        _: Player,
        _: View,
        _: Vec<(Move, Option<Ans>)>,
    ) -> Move {
        use proconio::input;

        println!("your turn");
        loop {
            input! {
                move_string: String,
            }

            if move_string == "Q" {
                input! {
                    pl_to: usize,
                    sort: String,
                }
                let Some(sort) = config.all_sort().into_iter().find(|s: &Sort| s.0 == sort) else {
                    println!("sort が正しく入力されなかった。");
                    continue;
                };
                return Move::Query {
                    query_to: config.player_turn(pl_to),
                    query_sort: sort,
                };
            } else if move_string == "A" {
                // o A x B x C で head が A, not B, not C の宣言
                input! {
                    sorts: [(char, String); config.all_sort().len()],
                }
                let declare: Vec<_> = sorts
                    .into_iter()
                    .map(|(c, sort)| (Sort(sort), c == 'o'))
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
    fn which(
        &mut self,
        config: GameConfig,
        player: Player,
        view: View,
        history: Vec<(Move, Option<Ans>)>,
    ) -> Move {
        let answerable = answerable(config.clone(), player, view.clone(), history.clone());

        if let Some(answer) = answerable {
            Move::Declare { declare: answer }
        } else {
            let my_query: HashSet<_> = history
                .iter()
                .skip(player.0)
                .step_by(3)
                .map(|(q, _)| q)
                .collect();
            let possible_query: Vec<_> = all_query(&config)
                .filter(|query| {
                    !my_query.contains(query) && {
                        let Move::Query {
                            query_to,
                            query_sort: _,
                        } = query
                        else {
                            unreachable!();
                        };
                        *query_to != player
                    }
                }) // TODO
                .collect();
            let n = self.thread_rng.gen_range(0..possible_query.len());
            if let Some(query) = possible_query.into_iter().nth(n) {
                query
            } else {
                // 聞くことができないので答えるしかない。
                let possible = possible_states(config.clone(), player, view, history)
                    .next()
                    .unwrap();
                let head = possible.players_head(player);
                Move::Declare {
                    declare: config
                        .all_sort()
                        .into_iter()
                        .map(|s| (s.clone(), config.has_sort(&head, &s)))
                        .collect(),
                }
            }
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
    fn which(
        &mut self,
        config: GameConfig,
        player: Player,
        view: View,
        history: Vec<(Move, Option<Ans>)>,
    ) -> Move {
        let answerable = answerable(config.clone(), player, view.clone(), history.clone());

        if let Some(answer) = answerable {
            Move::Declare { declare: answer }
        } else {
            let possible_states =
                possible_states(config.clone(), player, view, history).collect::<Vec<_>>();
            let possible_query = all_query(&config);

            let a = possible_query
                .filter_map(|query| {
                    let mut distribution: Vec<usize> = vec![0; config.cards_num() / 2 + 1];
                    // query に対して、 ans::QueAns(n) が返ってきたときの、それと整合する states の数の分布を表す: distribution[n] = #{state | 整合する}
                    for state in &possible_states {
                        let Ans::QueAns(n) = answer(&config, state, player, &query)? else {
                            unreachable!()
                        };
                        distribution[n] += 1;
                    }

                    let n: usize = distribution.iter().sum();
                    let mut entropy: f64 = 0_f64;
                    for i in distribution {
                        if i == 0 {
                            continue;
                        }
                        entropy += ((i as f64) / (n as f64)) * (i as f64).log2();
                    }

                    Some((query, entropy))
                })
                .min_by(|(_, entropy1), (_, entropy2)| entropy1.partial_cmp(entropy2).unwrap())
                .unwrap();

            a.0
        }
    }
}
