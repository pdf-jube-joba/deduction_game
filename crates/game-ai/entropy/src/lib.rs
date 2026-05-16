use std::collections::HashMap;

use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    defs::*,
    utils::*,
};

// 現在の履歴から可能な状態の全体を考え、各 query に対して可能な状態の回答の分布のエントロピーを計算する。
// 一番エントロピーが低いと、情報量がより得られているので、その手を選ぶ。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UseEntropyPlayer;

pub fn entoropy(info: Info) -> Option<Move> {
    let who = info.player_turn();
    let distrs: Vec<_> = possible_states(&info.config, &info.query_answer, &info.view).collect();

    info.movable_query()
        .into_iter()
        .filter_map(|q| {
            let mut distribution = vec![0; info.config.cards_num()];

            for distr in &distrs {
                let MoveAns::Query {
                    who: _,
                    query_to: _,
                    query_sort: _,
                    ans,
                } = answer(&info.config, distr, q.clone(), who)
                else {
                    unreachable!()
                };
                distribution[ans] += 1;
            }

            let mut k = 0;
            let mut entropy: f64 = 0_f64;
            for i in distribution {
                if i == 0 {
                    continue;
                }
                k += 1;
                entropy += ((i as f64) / (distrs.len() as f64)) * (i as f64).log2();
            }

            if k > 1 {
                Some((entropy, q))
            } else {
                None
            }
        })
        .min_by(|(entropy1, _), (entropy2, _)| entropy1.partial_cmp(entropy2).unwrap())
        .map(|(_, m)| m.clone())
}

impl Agent for UseEntropyPlayer {
    type Game = Game;

    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        _possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        if let Some(answer) = answerable_info(&info) {
            return answer;
        }

        let who = info.player_turn();
        let distrs: Vec<_> =
            possible_states(&info.config, &info.query_answer, &info.view).collect();

        debug_assert!(!distrs.is_empty());

        if let Some((_, q)) = info
            .movable_query()
            .into_iter()
            .filter_map(|q| {
                let mut distribution = vec![0; info.config.cards_num()];

                for distr in &distrs {
                    let MoveAns::Query {
                        who: _,
                        query_to: _,
                        query_sort: _,
                        ans,
                    } = answer(&info.config, distr, q.clone(), who)
                    else {
                        unreachable!()
                    };
                    distribution[ans] += 1;
                }

                let mut k = 0;
                let mut entropy: f64 = 0_f64;
                for i in distribution {
                    if i == 0 {
                        continue;
                    }
                    k += 1;
                    entropy += ((i as f64) / (distrs.len() as f64)) * (i as f64).log2();
                }

                if k > 1 {
                    Some((entropy, q))
                } else {
                    None
                }
            })
            .min_by(|(entropy1, _), (entropy2, _)| entropy1.partial_cmp(entropy2).unwrap())
        {
            q
        } else {
            let possible_declare = info.movable_declare();
            let mut maps: HashMap<Move, usize> = HashMap::new();
            for distr in distrs {
                let head = Move::Declare {
                    declare: players_head(&distr, who).clone(),
                };
                if !possible_declare.contains(&head) {
                    continue;
                }
                let entry = maps.entry(head);
                entry.and_modify(|i| *i += 1).or_default();
            }
            maps.into_iter().max_by_key(|(_, n)| *n).unwrap().0
        }
    }
}
