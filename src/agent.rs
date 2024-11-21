use std::collections::{HashMap, HashSet};

use super::{defs::*, utils::*};
use crate::abstract_game::{Agent, ImperfectInfoGame};
use rand::{
    rngs::{SmallRng, ThreadRng},
    thread_rng,
};

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
                let mut declare = HashSet::new();
                for _ in 0..info.config.head_num() {
                    input! {
                        n: usize,
                    }
                    declare.insert(Card(n));
                }
                return Move::Declare {
                    declare: declare.into_iter().collect(),
                };
            }
        }
    }
}

// 必ず当てれるときは当てるがそうじゃないときは可能な手からランダムに打つ。
#[derive(Debug, Clone, PartialEq)]
pub struct RandomPlayer<R>
where
    R: rand::Rng,
{
    rng: R,
}

impl<R> RandomPlayer<R>
where
    R: rand::Rng,
{
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl Default for RandomPlayer<ThreadRng> {
    fn default() -> Self {
        Self { rng: thread_rng() }
    }
}

impl<R> Agent for RandomPlayer<R>
where
    R: rand::Rng,
{
    type Game = Game;
    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        _possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        // answerable なとき
        if let Some(answer) = answerable_info(&info) {
            return answer;
        }
        let possible_moves = info.movable_query();
        if possible_moves.is_empty() {
            let possible_declare = info.movable_declare();
            possible_declare.into_iter().next().unwrap() // possible declare がないのはありえないと思う。
        } else {
            random_vec(&mut self.rng, possible_moves.into_iter().collect())
        }
    }
}

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
            // この質問をして意味があるか
            let mut k = 0;

            let mut entropy: f64 = 0_f64;
            for i in distribution {
                if i == 0 {
                    continue;
                }
                k += 1;
                entropy += ((i as f64) / (distrs.len() as f64)) * (i as f64).log2();
            }

            // k > 1 なら質問をすると分類ができるが、できないものは質問しても仕方ないので選択肢から省く。
            if k > 1 {
                Some((entropy, q))
            } else {
                None
            }
        })
        .min_by(|(entropy1, _), (entropy2, _)| entropy1.partial_cmp(entropy2).unwrap())
        .map(|(_, m)| m.clone())
}

// 現在の履歴から可能な状態の全体を考え、各 query に対して可能な状態の回答の分布のエントロピーを計算する。
// 一番エントロピーが低いと、情報量がより得られているので、その手を選ぶ。
#[derive(Debug, Clone, PartialEq)]
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
                // この質問をして意味があるか
                let mut k = 0;

                let mut entropy: f64 = 0_f64;
                for i in distribution {
                    if i == 0 {
                        continue;
                    }
                    k += 1;
                    entropy += ((i as f64) / (distrs.len() as f64)) * (i as f64).log2();
                }

                // k > 1 なら質問をすると分類ができるが、できないものは質問しても仕方ないので選択肢から省く。
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
            // 聞けることすべて聞いて、特定ができてないケース
            let possible_declare = info.movable_declare();
            // 一番ありうる頭のカードを当てるためにカウントをとる
            let mut maps: HashMap<Move, usize> = HashMap::new();
            for distr in distrs {
                let head = Move::Declare {
                    declare: distr.players_head(who).clone(),
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

pub fn search_depth(info: &Info, depth: usize) -> Option<(Move, Vec<f64>)> {
    let mut query_answer = info.query_answer.clone();
    let movables = info
        .config
        .all_player()
        .into_iter()
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
        for m in &movables[usize::from(now_player)] {
            let mut points = vec![0_f64; player_num];
            for distr in &possible_state {
                let ans = answer(config, distr, m.clone(), now_player);
                query_answer.push(ans);
                let view = distr.cards_from_player(next_player);
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

            for v in points.iter_mut() {
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
        possible_moves.into_iter().nth(0).unwrap()
    }
}

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

#[derive(Debug, Clone)]
#[cfg_attr(target_family = "wasm", derive(PartialEq))]
pub enum Opponent {
    Entoropy(UseEntropyPlayer),
    #[cfg(target_arch = "x86_64")]
    RandomThreadRng(RandomPlayer<ThreadRng>),
    RandomSmallRng(RandomPlayer<SmallRng>),
    SearchPlayer(SearchPlayer),
    Unfair(Unfair),
}

impl Agent for Opponent {
    type Game = Game;
    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        match self {
            Opponent::Entoropy(p) => p.use_info(info, possible_moves),
            #[cfg(target_arch = "x86_64")]
            Opponent::RandomThreadRng(p) => p.use_info(info, possible_moves),
            Opponent::RandomSmallRng(p) => p.use_info(info, possible_moves),
            Opponent::SearchPlayer(p) => p.use_info(info, possible_moves),
            Opponent::Unfair(p) => p.use_info(info, possible_moves),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a() {
        let config = default_config();
        let mut game = config.gen_random(&mut thread_rng());
        game.move_game(Move::Query {
            query_to: 1.into(),
            query_sort: "A".into(),
        });
        game.move_game(Move::Query {
            query_to: 2.into(),
            query_sort: "A".into(),
        });
        game.move_game(Move::Query {
            query_to: 2.into(),
            query_sort: "B".into(),
        });
        let info = game.info_and_move_now();
        let a = search_depth(&info.0, 4);
        eprintln!("{a:?}")
    }
}
