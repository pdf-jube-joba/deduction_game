use super::defs::*;
use itertools::Itertools;
use std::collections::HashSet;

pub fn default_config() -> GameConfig {
    crate::defs::GameConfig::new(
        vec!["A", "B", "X", "Y", "Z"]
            .into_iter()
            .map(|str| Sort(str.to_string()))
            .collect(),
        6,
        vec![
            (0, "A"),
            (1, "A"),
            (2, "A"),
            (3, "B"),
            (4, "B"),
            (5, "B"),
            (0, "X"),
            (1, "Y"),
            (2, "Z"),
            (3, "X"),
            (4, "Y"),
            (5, "Z"),
        ]
        .into_iter()
        .map(|(i, s)| (Card(i), Sort(s.to_string())))
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
) -> Option<Vec<Vec<Sort>>> {
    let player = config.player_turn(query_answer.len());
    let possible_distr = possible_states(Info {
        config: config.clone(),
        query_answer: query_answer.clone(),
        view: view.clone(),
    });
    let heads: Vec<_> = possible_distr
        .into_iter()
        .map(|distr| distr.players_head(player))
        .collect();
    if heads.is_empty() {
        return None;
    }
    let mut heads_sorts = vec![];
    for i in 0..config.head_num() {
        let mut sets = heads.iter().map(|head| {
            config
                .all_sort_of_card(&head[i])
                .into_iter()
                .collect::<HashSet<_>>()
        });
        let first = sets.next().unwrap();
        if sets.any(|s| s != first) {
            return None;
        }
        heads_sorts.push(first.into_iter().collect());
    }
    Some(heads_sorts)
}

pub fn query_at(
    Info {
        config,
        query_answer,
        view: _,
    }: Info,
) -> Vec<Move> {
    let player = config.player_turn(query_answer.len());
    let mut all_query: HashSet<_> = all_query(&config).collect();
    for qa in query_answer
        .iter()
        .skip(player)
        .step_by(config.player_num())
    {
        all_query.remove(&qa.move_of_this());
    }
    all_query.into_iter().collect()
}

pub fn random_vec<R, T>(rng: &mut R, v: Vec<T>) -> T
where
    R: rand::Rng,
{
    let i = rng.gen_range(0..v.len());
    v.into_iter().nth(i).unwrap()
}

pub fn declare_at(
    Info {
        config,
        query_answer,
        view: _,
    }: Info,
) -> impl Iterator<Item = Move> {
    let all_q: HashSet<_> = query_answer
        .into_iter()
        .map(|qa| qa.move_of_this())
        .collect();
    config
        .all_cards()
        .into_iter()
        .permutations(config.head_num())
        .map(move |v| {
            let declare: Vec<Vec<Sort>> =
                v.into_iter().map(|c| config.all_sort_of_card(&c)).collect();
            Move::Declare { declare }
        })
        .filter(move |q| !all_q.contains(q))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_config() {
        let config = default_config();
        eprintln!("{config:?}");
    }
}
