use crate::abstract_game::{self, Player};
use itertools::Itertools;
use std::{collections::HashSet, fmt::Display};

// type of sort
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sort(pub String);

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Card(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GameConfig {
    sort_kinds: Vec<Sort>,
    cards_num: usize,
    cards_sort: Vec<(Card, Sort)>,
    player_num: usize,
    head_num: usize,
    hand_num: usize,
}

impl GameConfig {
    pub fn new(
        sort_kinds: Vec<Sort>,
        cards_num: usize,
        cards_sort: Vec<(Card, Sort)>,
        player_num: usize,
        head_num: usize,
        hand_num: usize,
    ) -> Option<Self> {
        if (head_num + hand_num) * player_num > cards_num {
            return None;
        }
        for (_, s) in &cards_sort {
            if !sort_kinds.contains(s) {
                return None;
            }
        }
        Some(Self {
            sort_kinds,
            cards_num,
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
        n % self.player_num
    }
    pub fn cards_num(&self) -> usize {
        self.cards_num
    }
    pub fn head_num(&self) -> usize {
        self.head_num
    }
    pub fn hand_num(&self) -> usize {
        self.hand_num
    }
    pub fn all_player(&self) -> Vec<Player> {
        (0..self.player_num).collect()
    }
    pub fn all_sort(&self) -> Vec<Sort> {
        self.sort_kinds.clone()
    }
    pub fn all_cards(&self) -> Vec<Card> {
        (0..self.cards_num).map(Card).collect()
    }
    pub fn all_sort_of_card(&self, card: &Card) -> Vec<Sort> {
        self.cards_sort
            .iter()
            .filter_map(|(c, s)| if c == card { Some(s.clone()) } else { None })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlCard {
    pub hand: Vec<Card>,
    pub head: Vec<Card>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Distr {
    state: Vec<PlCard>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct View {
    pub hand: Vec<Card>,
    pub other: Vec<Option<Vec<Card>>>,
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
        self.cards_sort.iter().any(|(c, s)| c == card && s == sort)
    }
    pub fn all_states(&self) -> Vec<Distr> {
        self.all_cards()
            .into_iter()
            .permutations(self.cards_num())
            .map(|perm| {
                let mut state = vec![];
                for p in 0..self.player_num {
                    let ind = (self.head_num + self.hand_num) * p;
                    state.push(PlCard {
                        hand: perm[ind..ind + self.hand_num].to_vec(),
                        head: perm[ind + self.hand_num..ind + self.hand_num + self.head_num]
                            .to_vec(),
                    })
                }
                Distr { state }
            })
            .collect()
    }
    pub fn gen_random<R>(&self, rng: &mut R) -> Game
    where
        R: rand::Rng,
    {
        let all_states = self.all_states();
        let ind = rng.gen_range(0..all_states.len());
        let distr = all_states.into_iter().nth(ind).unwrap();
        Game {
            config: self.clone(),
            distr,
            query_answer: vec![],
        }
    }
}

pub fn answer(config: &GameConfig, distr: &Distr, m: Move, who: Player) -> MoveAns {
    debug_assert!(config.all_states().contains(distr));
    match m {
        Move::Query {
            query_to,
            query_sort,
        } => {
            let view = distr.cards_from_player(query_to);
            let sort_num: usize = view.sort_num(config, &query_sort);
            MoveAns::Query {
                query_to,
                query_sort,
                ans: sort_num,
            }
        }
        Move::Declare { declare } => {
            let player_head = distr.players_head(who);
            let ans = (0..config.head_num).all(|i| {
                let set1: HashSet<_> = declare[i].iter().cloned().collect();
                let card = player_head[i];
                let set2: HashSet<_> = config.all_sort_of_card(&card).into_iter().collect();
                set1 == set2
            });
            MoveAns::Declare { declare, ans }
        }
    }
}

impl Distr {
    pub fn players_head(&self, player: Player) -> Vec<Card> {
        self.state[player].head.clone()
    }

    pub fn players_hand(&self, player: Player) -> Vec<Card> {
        self.state[player].hand.clone()
    }

    pub fn cards_from_player(&self, player: Player) -> View {
        let hand = self.players_hand(player);
        let other = self
            .state
            .iter()
            .enumerate()
            .map(|(i, c)| {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Move {
    Query { query_to: Player, query_sort: Sort }, // 同じ質問はできない。
    Declare { declare: Vec<Vec<Sort>> },          // 全てのソートについて回答している必要がある。
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MoveAns {
    Query {
        query_to: Player,
        query_sort: Sort,
        ans: usize,
    },
    Declare {
        declare: Vec<Vec<Sort>>,
        ans: bool,
    },
}

impl MoveAns {
    pub fn move_of_this(&self) -> Move {
        match self {
            MoveAns::Query {
                query_to,
                query_sort,
                ans: _,
            } => Move::Query {
                query_to: *query_to,
                query_sort: query_sort.clone(),
            },
            MoveAns::Declare { declare, ans: _ } => Move::Declare {
                declare: declare.clone(),
            },
        }
    }
}

pub fn all_query(config: &GameConfig) -> impl Iterator<Item = Move> {
    itertools::iproduct!(config.all_player(), config.all_sort()).map(|(player_num, sort)| {
        Move::Query {
            query_to: player_num,
            query_sort: sort,
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    config: GameConfig,
    distr: Distr,
    query_answer: Vec<MoveAns>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub config: GameConfig,
    pub query_answer: Vec<MoveAns>,
    pub view: View,
}

impl Info {
    pub fn player_turn(&self) -> Player {
        self.config.player_turn(self.query_answer.len())
    }
    pub fn query_at(&self) -> Vec<Move> {
        let p = self.config.player_turn(self.query_answer.len());
        let past_moves: HashSet<Move> = self
            .query_answer
            .iter()
            .skip(p)
            .step_by(self.config.player_num())
            .map(|qa| qa.move_of_this())
            .collect();
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
    pub fn declare_at(&self) -> Vec<Move> {
        let p = self.config.player_turn(self.query_answer.len());
        let past_moves: HashSet<Move> = self
            .query_answer
            .iter()
            .skip(p)
            .step_by(self.config.player_num())
            .map(|qa| qa.move_of_this())
            .collect();
        self.config
            .all_cards()
            .into_iter()
            .permutations(self.config.head_num())
            .map(move |v| {
                let declare: Vec<Vec<Sort>> = v
                    .into_iter()
                    .map(|c| self.config.all_sort_of_card(&c))
                    .collect();
                Move::Declare { declare }
            })
            .inspect(|d| eprintln!("p:{d:?}"))
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
        eprintln!("{:?}", self.is_win());
        if self.is_win().is_some() {
            return (info, vec![]);
        }
        let m = info
            .query_at()
            .into_iter()
            .chain(info.declare_at())
            .collect();
        (info, m)
    }

    fn is_win(&self) -> Option<Vec<usize>> {
        let p = self.player_turn();
        let mut v = vec![0; self.player_number()];
        if let Some(MoveAns::Declare {
            declare: _,
            ans: true,
        }) = self.query_answer.last()
        {
            v[p] = 1;
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
