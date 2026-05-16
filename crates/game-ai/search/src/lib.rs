use std::collections::HashSet;

use game_core::{
    abstract_game::{Agent, ImperfectInfoGame},
    defs::*,
    utils::*,
};

pub fn search_depth(info: &Info, depth: usize) -> Option<(Move, Vec<f64>)> {
    let mut query_answer = info.query_answer.clone();
    let movables = (0..info.config.player_num())
        .map(|player| movable_query_ref(&info.config, &query_answer, player).collect())
        .collect();
    search_rec(
        &info.config,
        &mut query_answer,
        &info.view,
        depth,
        &movables,
    )
}

pub fn search_rec(
    config: &GameConfig,
    query_answer: &mut Vec<MoveAns>,
    view: &View,
    depth: usize,
    movables: &Vec<HashSet<Move>>,
) -> Option<(Move, Vec<f64>)> {
    let player_num = config.player_num();
    if let Some(answer) = answerable(config, query_answer, view) {
        let mut v = vec![0_f64; player_num];
        v[0] = 1_f64;
        return Some((answer, v));
    }

    if depth == 0 {
        None
    } else {
        let now_player = config.player_turn(query_answer.len());
        let next_player = config.player_turn(query_answer.len() + 1);
        let possible_state: Vec<_> = possible_states(config, query_answer, view).collect();
        let state_len = possible_state.len();

        let mut min: Option<(&Move, Vec<f64>)> = None;
        for m in &movables[now_player] {
            let mut points = vec![0_f64; player_num];
            for distr in &possible_state {
                let ans = answer(config, distr, m.clone(), now_player);
                query_answer.push(ans);
                let view = cards_from_player(distr, next_player);
                let res = search_rec(config, query_answer, &view, depth - 1, movables);
                query_answer.pop();
                let Some((_, mut point)) = res else {
                    continue;
                };
                point.rotate_right(1);
                for i in 0..player_num {
                    points[i] += point[i];
                }
            }

            for v in &mut points {
                *v /= state_len as f64;
            }

            match &min {
                None => min = Some((m, points)),
                Some(x) => {
                    if x.1[0] <= points[0] {
                        min = Some((m, points))
                    }
                }
            }
        }

        min.map(|(m, v)| (m.clone(), v))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchPlayer {
    depth: usize,
}

impl SearchPlayer {
    // depth >= 5 はあまりにも時間を使うので危険
    pub fn new(depth: usize) -> SearchPlayer {
        SearchPlayer { depth }
    }
}

impl Agent for SearchPlayer {
    type Game = Game;

    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        if let Some(answer) = answerable_info(&info) {
            return answer;
        }
        if let Some((m, _)) = search_depth(&info, self.depth) {
            return m;
        }
        possible_moves.into_iter().next().unwrap()
    }
}
