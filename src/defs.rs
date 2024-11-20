use crate::abstract_game::{self, Player};
use itertools::Itertools;
use std::{
    collections::BTreeSet,
    fmt::Display,
    ops::{Index, IndexMut},
};

// type of sort
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sort(pub String);

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Card(i) があるなら j < i に対して Card(j) もあること
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card(pub usize);

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0 + 1)
    }
}

impl From<usize> for Card {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<Card> for usize {
    fn from(value: Card) -> Self {
        value.0
    }
}

impl<T> Index<Card> for Vec<T> {
    type Output = T;
    fn index(&self, index: Card) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> IndexMut<Card> for Vec<T> {
    fn index_mut(&mut self, index: Card) -> &mut Self::Output {
        &mut self[index.0]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameConfig {
    sorts: BTreeSet<Sort>,
    cards_sort: Vec<BTreeSet<Sort>>,
    player_num: usize,
    head_num: usize,
    hand_num: usize,
}

impl GameConfig {
    pub fn new(
        sorts: BTreeSet<Sort>,
        cards_sort: Vec<BTreeSet<Sort>>, // cards_sort[i] = i-th card's sorts
        player_num: usize,
        head_num: usize,
        hand_num: usize,
    ) -> Option<Self> {
        let cards_num = cards_sort.len();
        if (head_num + hand_num) * player_num > cards_num {
            return None;
        }
        for ss in &cards_sort {
            for s in ss.into_iter() {
                if !sorts.contains(s) {
                    return None;
                }
            }
        }
        Some(Self {
            sorts,
            cards_sort,
            player_num,
            head_num,
            hand_num,
        })
    }
    pub fn player_num(&self) -> usize {
        self.player_num
    }
    // input turn: usize => which player should move
    pub fn player_turn(&self, n: usize) -> Player {
        (n % self.player_num).into()
    }
    pub fn cards_num(&self) -> usize {
        self.cards_sort.len()
    }
    pub fn head_num(&self) -> usize {
        self.head_num
    }
    pub fn hand_num(&self) -> usize {
        self.hand_num
    }
    pub fn all_player(&self) -> Vec<Player> {
        (0..self.player_num).map(|i| i.into()).collect()
    }
    pub fn all_sort(&self) -> BTreeSet<Sort> {
        self.sorts.clone()
    }
    pub fn all_cards(&self) -> Vec<Card> {
        (0..self.cards_num()).map(Card).collect()
    }
    pub fn all_sort_of_card(&self, card: &Card) -> &BTreeSet<Sort> {
        if card.0 > self.cards_num() {
            panic!("変なカード")
        }
        &self.cards_sort[card.0]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlCard {
    pub hand: BTreeSet<Card>,
    pub head: BTreeSet<Card>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Distr {
    state: Vec<PlCard>, // state[i] = player i's hand and head
}

#[derive(Debug, Clone, PartialEq)]
pub struct View {
    pub hand: BTreeSet<Card>,
    pub other: Vec<Option<BTreeSet<Card>>>,
}

impl View {
    fn sort_num(&self, config: &GameConfig, s: &Sort) -> usize {
        self.other
            .iter()
            .filter_map(|v| v.as_ref())
            .flatten()
            .chain(self.hand.iter())
            .filter(|c| config.has_sort(c, s))
            .count()
    }
}

impl GameConfig {
    pub fn has_sort(&self, card: &Card, sort: &Sort) -> bool {
        self.cards_sort[card.0].contains(sort)
    }
    pub fn gen_random<R>(&self, rng: &mut R) -> Game
    where
        R: rand::Rng,
    {
        let mut perm = self.all_cards();
        let len = perm.len();
        for _ in 0..self.cards_num().pow(2) {
            let i = rng.gen_range(0..len);
            let j = rng.gen_range(0..len);
            perm.swap(i, j);
        }

        let mut perm = perm.into_iter();

        let mut perm_consume = |i: usize| -> BTreeSet<Card> {
            let mut v = BTreeSet::new();
            for _ in 0..i {
                v.insert(perm.next().unwrap());
            }
            v
        };

        let mut state = vec![];
        for _ in 0..self.player_num {
            state.push(PlCard {
                hand: perm_consume(self.hand_num),
                head: perm_consume(self.head_num),
            })
        }
        Game {
            config: self.clone(),
            distr: Distr { state },
            query_answer: vec![],
        }
    }
}

impl Distr {
    pub fn new(state: Vec<PlCard>) -> Self {
        Self { state }
    }
    pub fn players_head(&self, player: Player) -> &BTreeSet<Card> {
        let i: usize = player.into();
        &self.state[i].head
    }

    pub fn players_hand(&self, player: Player) -> &BTreeSet<Card> {
        let i: usize = player.into();
        &self.state[i].hand
    }

    pub fn cards_from_player(&self, player: Player) -> View {
        let hand = self.players_hand(player).clone();
        let other = self
            .state
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let i: Player = i.into();
                if i != player {
                    Some(c.head.clone())
                } else {
                    None
                }
            })
            .collect();
        View { hand, other }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Move {
    Query { query_to: Player, query_sort: Sort }, // 同じ質問はできない。
    Declare { declare: BTreeSet<Card> },          // 全てのソートについて回答している必要がある。
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MoveAns {
    Query {
        who: Player,
        query_to: Player,
        query_sort: Sort,
        ans: usize,
    },
    Declare {
        who: Player,
        declare: BTreeSet<Card>,
        ans: bool,
    },
}

impl MoveAns {
    pub fn move_of_this(&self) -> Move {
        match self {
            MoveAns::Query {
                who: _,
                query_to,
                query_sort,
                ans: _,
            } => Move::Query {
                query_to: *query_to,
                query_sort: query_sort.clone(),
            },
            MoveAns::Declare {
                who: _,
                declare,
                ans: _,
            } => Move::Declare {
                declare: declare.clone(),
            },
        }
    }
}

pub fn answer(config: &GameConfig, distr: &Distr, m: Move, who: Player) -> MoveAns {
    match m {
        Move::Query {
            query_to,
            query_sort,
        } => {
            let view = distr.cards_from_player(query_to);
            let sort_num: usize = view.sort_num(config, &query_sort);
            MoveAns::Query {
                who,
                query_to,
                query_sort,
                ans: sort_num,
            }
        }
        Move::Declare { declare } => {
            let player_head = distr.players_head(who);
            let b = declare.iter().cloned().collect::<BTreeSet<_>>() == *player_head;
            MoveAns::Declare {
                who,
                declare,
                ans: b,
            }
        }
    }
}

pub fn all_query(config: &GameConfig) -> impl Iterator<Item = Move> {
    itertools::iproduct!(config.all_sort(), config.all_player(),).map(|(sort, player_num)| {
        Move::Query {
            query_to: player_num,
            query_sort: sort,
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct History(Vec<MoveAns>);

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    config: GameConfig,
    distr: Distr,
    query_answer: Vec<MoveAns>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub config: GameConfig,
    pub query_answer: Vec<MoveAns>,
    pub view: View,
}

impl Info {
    pub fn player_turn(&self) -> Player {
        self.config.player_turn(self.query_answer.len())
    }
    pub fn moves_of_player(&self, player: Player) -> Vec<Move> {
        let player: usize = player.into();
        self.query_answer
            .iter()
            .skip(player)
            .step_by(self.config.player_num())
            .map(|qa| qa.move_of_this())
            .collect()
    }
    pub fn movable_query(&self) -> BTreeSet<Move> {
        let p = self.config.player_turn(self.query_answer.len());
        let past_moves = self.moves_of_player(p);
        all_query(&self.config)
            .filter(|q| {
                !past_moves.contains(q)
                    && !matches!(
                        q,
                        Move::Query {
                            query_to,
                            query_sort: _
                        } if *query_to == p
                    )
            })
            .collect()
    }
    pub fn movable_declare(&self) -> BTreeSet<Move> {
        let p = self.config.player_turn(self.query_answer.len());
        let past_moves: Vec<Move> = self.moves_of_player(p);
        self.config
            .all_cards()
            .into_iter()
            .permutations(self.config.head_num())
            .map(move |declare| Move::Declare {
                declare: declare.into_iter().collect(),
            })
            .filter(move |q| !past_moves.contains(q))
            .collect()
    }
}

impl abstract_game::ImperfectInfoGame for Game {
    type Info = Info;
    type Move = Move;
    fn player_number(&self) -> usize {
        self.config.player_num
    }
    fn player_turn(&self) -> Player {
        self.config.player_turn(self.query_answer.len())
    }
    fn info_and_move_now(&self) -> (Self::Info, Vec<Self::Move>) {
        let info = Self::Info {
            config: self.config.clone(),
            query_answer: self.query_answer.clone(),
            view: self.distr.cards_from_player(self.player_turn()),
        };
        if self.is_win().is_some() {
            return (info, vec![]);
        }

        let m = info
            .movable_query()
            .into_iter()
            .chain(info.movable_declare())
            .collect();
        (info, m)
    }

    fn is_win(&self) -> Option<Vec<usize>> {
        if self.query_answer.is_empty() {
            return None;
        }
        let mut v = vec![0; self.player_number()];
        if let Some(MoveAns::Declare {
            who,
            declare: _,
            ans: true,
        }) = self.query_answer.last()
        {
            let who: usize = (*who).into();
            v[who] = 1;
            Some(v)
        } else {
            None
        }
    }
    fn move_game(&mut self, m: Self::Move) -> bool {
        if !self.info_and_move_now().1.contains(&m) {
            return false;
        }
        let qa = answer(&self.config, &self.distr, m, self.player_turn());
        self.query_answer.push(qa);
        true
    }
}
