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

pub fn possible_states(
    Info {
        config,
        query_answer,
        view,
    }: Info,
) -> impl Iterator<Item = Distr> {
    let player = config.player_turn(query_answer.len());
    config.all_states().into_iter().filter(move |distr| {
        let distr_view = distr.cards_from_player(player);
        view == distr_view
            && query_answer.iter().enumerate().all(|(turn, qa)| {
                let m = qa.move_of_this();
                let who = config.player_turn(turn);
                *qa == answer(&config, distr, m, who)
            })
    })
}

pub fn answerable(
    Info {
        config,
        query_answer,
        view,
    }: Info,
) -> Option<BTreeSet<Card>> {
    let player = config.player_turn(query_answer.len());
    let possible_distr = possible_states(Info {
        config: config.clone(),
        query_answer: query_answer.clone(),
        view: view.clone(),
    });
    let mut heads = possible_distr
        .into_iter()
        .map(|distr| distr.players_head(player).clone());
    let Some(head) = heads.next() else {
        unreachable!("頭にちゃんとカードはあるはず");
    };
    if heads.next().is_some() {
        None
    } else {
        Some(head)
    }
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
        eprintln!("{config:?}");
        let game = config.gen_random(&mut thread_rng());
        eprintln!("{game:?}");
        let info = game.info_and_move_now();
        eprintln!("{info:?}");
    }
}
