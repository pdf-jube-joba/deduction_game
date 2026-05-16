use std::collections::BTreeSet;

use crate::defs::GameConfig;

pub fn default_config() -> GameConfig {
    GameConfig::new(
        vec!["A", "B", "X", "Y", "Z"]
            .into_iter()
            .map(|str| str.to_string())
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
        .map(|v| v.into_iter().map(|s| s.to_string()).collect())
        .collect(),
        3,
        1,
        1,
    )
    .unwrap()
}

pub fn three_midium() -> GameConfig {
    GameConfig::new(
        vec!["A", "B", "C", "X", "Y", "Z", "W"]
            .into_iter()
            .map(|str| str.to_string())
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
        .map(|v| v.into_iter().map(|s| s.to_string()).collect())
        .collect(),
        3,
        2,
        2,
    )
    .unwrap()
}

pub fn four_midium() -> GameConfig {
    GameConfig::new(
        vec!["A", "B", "C", "X", "Y", "Z", "W"]
            .into_iter()
            .map(|str| str.to_string())
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
        .map(|v| v.into_iter().map(|s| s.to_string()).collect())
        .collect(),
        4,
        2,
        1,
    )
    .unwrap()
}

pub fn from_sorts_and_cards(
    sorts: impl IntoIterator<Item = &'static str>,
    cards_sort: impl IntoIterator<Item = impl IntoIterator<Item = &'static str>>,
    player_num: usize,
    head_num: usize,
    hand_num: usize,
) -> Option<GameConfig> {
    GameConfig::new(
        sorts
            .into_iter()
            .map(|sort| sort.to_string())
            .collect::<BTreeSet<_>>(),
        cards_sort
            .into_iter()
            .map(|sorts| sorts.into_iter().map(|sort| sort.to_string()).collect())
            .collect(),
        player_num,
        head_num,
        hand_num,
    )
}
