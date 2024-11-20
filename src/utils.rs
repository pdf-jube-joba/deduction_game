use itertools::Itertools;

use crate::abstract_game::Player;

use super::defs::*;
use std::collections::{BTreeSet, HashSet};

pub fn default_config() -> GameConfig {
    crate::defs::GameConfig::new(
        vec!["A", "B", "X", "Y", "Z"]
            .into_iter()
            .map(|str| Sort(str.to_string()))
            .collect(),
        vec![
            vec!["A", "X"],
            vec!["A", "Y"],
            vec!["A", "Z"],
            vec!["B", "X"],
            vec!["B", "Y"],
            vec!["B", "Z"],
        ]
        .into_iter()
        .map(|v| v.into_iter().map(|s| Sort(s.to_string())).collect())
        .collect(),
        3,
        1,
        1,
    )
    .unwrap()
}

pub fn three_midium() -> GameConfig {
    crate::defs::GameConfig::new(
        vec!["A", "B", "C", "X", "Y", "Z", "W"]
            .into_iter()
            .map(|str| Sort(str.to_string()))
            .collect(),
        vec![
            vec!["A", "X"],
            vec!["B", "X"],
            vec!["C", "X"],
            vec!["A", "Y"],
            vec!["B", "Y"],
            vec!["C", "Y"],
            vec!["A", "Z"],
            vec!["B", "Z"],
            vec!["C", "Z"],
            vec!["A", "W"],
            vec!["B", "W"],
            vec!["C", "W"],
        ]
        .into_iter()
        .map(|v| v.into_iter().map(|s| Sort(s.to_string())).collect())
        .collect(),
        3,
        2,
        2,
    )
    .unwrap()
}

pub fn four_midium() -> GameConfig {
    crate::defs::GameConfig::new(
        vec!["A", "B", "C", "X", "Y", "Z", "W"]
            .into_iter()
            .map(|str| Sort(str.to_string()))
            .collect(),
        vec![
            vec!["A", "X"],
            vec!["B", "X"],
            vec!["C", "X"],
            vec!["A", "Y"],
            vec!["B", "Y"],
            vec!["C", "Y"],
            vec!["A", "Z"],
            vec!["B", "Z"],
            vec!["C", "Z"],
            vec!["A", "W"],
            vec!["B", "W"],
            vec!["C", "W"],
        ]
        .into_iter()
        .map(|v| v.into_iter().map(|s| Sort(s.to_string())).collect())
        .collect(),
        4,
        2,
        1,
    )
    .unwrap()
}

pub fn possible_states<'a>(
    config: &'a GameConfig,
    query_answer: &'a Vec<MoveAns>,
    view: &'a View,
) -> impl Iterator<Item = Distr> + 'a {
    let player = config.player_turn(query_answer.len());
    let not_in_view: Vec<Card> = config
        .all_cards()
        .into_iter()
        .filter(|c| {
            !view.hand.contains(c)
                && view
                    .other
                    .iter()
                    .filter_map(|s| s.as_ref())
                    .all(|v| !v.contains(c))
        })
        .collect();

    let n = not_in_view.len();
    let all_player = config.all_player();
    let (head_num, hand_num) = (config.head_num(), config.hand_num());

    not_in_view
        .into_iter()
        .permutations(n)
        .map(move |not_in_view| {
            let mut state = vec![];
            let mut perm = not_in_view.into_iter();

            let mut perm_consume = |i: usize| -> BTreeSet<Card> {
                let mut v = BTreeSet::new();
                for _ in 0..i {
                    v.insert(perm.next().unwrap());
                }
                v
            };

            for p in all_player.clone() {
                let (head, hand) = if p == player {
                    (perm_consume(head_num), view.hand.clone())
                } else {
                    let p: usize = p.into();
                    (
                        view.other[p].as_ref().unwrap().clone(),
                        perm_consume(hand_num),
                    )
                };
                state.push(PlCard { head, hand })
            }
            Distr::new(state)
        })
        .filter(move |distr| {
            query_answer.iter().all(|qa| {
                let who = match qa {
                    MoveAns::Query {
                        who,
                        query_to,
                        query_sort,
                        ans,
                    } => who,
                    MoveAns::Declare { who, declare, ans } => who,
                };
                let m = qa.move_of_this();
                let a = answer(config, distr, m, *who);
                *qa == a
            })
        })
}

pub fn movable_query_ref<'a>(
    config: &'a GameConfig,
    query_answer: &'a Vec<MoveAns>,
    player: Player,
) -> impl Iterator<Item = Move> + 'a {
    let past_moves: HashSet<_> = query_answer
        .iter()
        .skip(player.into())
        .step_by(config.player_num())
        .map(|qa| qa.move_of_this())
        .collect();
    all_query(config).filter(move |q| {
        !past_moves.contains(q)
            && !matches!(
                q,
                Move::Query {
                    query_to,
                    query_sort: _
                } if *query_to == player
            )
    })
}

pub fn answerable(config: &GameConfig, query_answer: &Vec<MoveAns>, view: &View) -> Option<Move> {
    let player = config.player_turn(query_answer.len());
    let possible_distr = possible_states(config, query_answer, view);
    let mut heads = possible_distr
        .into_iter()
        .map(|distr| distr.players_head(player).clone());
    let Some(head) = heads.next() else {
        unreachable!("頭にちゃんとカードはあるはず");
    };
    for other_head in heads {
        if other_head != head {
            return None;
        }
    }
    Some(Move::Declare { declare: head })
}

pub fn answerable_info(
    Info {
        config,
        query_answer,
        view,
    }: &Info,
) -> Option<Move> {
    answerable(config, query_answer, view)
}

pub fn random_vec<R, T>(rng: &mut R, v: Vec<T>) -> T
where
    R: rand::Rng,
{
    let i = rng.gen_range(0..v.len());
    v.into_iter().nth(i).unwrap()
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use crate::abstract_game::ImperfectInfoGame;

    use super::*;
    #[test]
    fn test() {
        let config = default_config();
        // eprintln!("{config:?}");
        let game = config.gen_random(&mut thread_rng());
        // eprintln!("{game:?}");
        let info = game.info_and_move_now().0;
        // eprintln!("{info:?}");
        let p = possible_states(&info.config, &info.query_answer, &info.view);
        for p in p {
            eprintln!("{:?}", p);
        }

        let _ = answerable(&info.config, &info.query_answer, &info.view);

        // let config = three_midium();
        // eprintln!("{config:?}");
        // let game = config.gen_random(&mut thread_rng());
        // eprintln!("{game:?}");
        // let info = game.info_and_move_now();
        // eprintln!("{info:?}");

        // let config = four_midium();
        // eprintln!("{config:?}");
        // let game = config.gen_random(&mut thread_rng());
        // eprintln!("{game:?}");
        // let info = game.info_and_move_now();
        // eprintln!("{info:?}");
    }
}
