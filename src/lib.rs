pub mod game {
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Player(pub usize);

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct GameConfig {
        sort_kinds: Vec<Sort>,
        cards_number: usize,
        cards_sort: Vec<(Card, Sort)>,
        player_number: usize,
    }

    impl GameConfig {
        pub fn player_num(&self) -> usize {
            self.player_number
        }
        pub fn cards_num(&self) -> usize {
            self.player_number * 2
        }
        // input turn: usize => which player should move
        pub fn player_turn(&self, n: usize) -> Player {
            Player(n % self.player_number)
        }
        pub fn all_player(&self) -> Vec<Player> {
            (0..self.player_number).map(Player).collect()
        }
        pub fn all_sort(&self) -> Vec<Sort> {
            self.sort_kinds.clone()
        }
        pub fn all_cards(&self) -> Vec<Card> {
            (0..self.cards_number).map(Card).collect()
        }
        pub fn all_sort_of_card(&self, card: &Card) -> Vec<Sort> {
            self.cards_sort
                .iter()
                .filter_map(|(c, s)| if c == card { Some(s.clone()) } else { None })
                .collect()
        }
        pub fn has_sort(&self, card: &Card, sort: &Sort) -> bool {
            self.cards_sort.iter().any(|(c, s)| c == card && s == sort)
        }
        pub fn all_states(&self) -> Vec<State> {
            self.all_cards()
                .into_iter()
                .permutations(self.cards_num())
                .map(|perm| {
                    let mut state = vec![];
                    for i in 0..(self.cards_num() / 2) {
                        state.push(PlCard {
                            hand: perm[2 * i],
                            head: perm[2 * i + 1],
                        })
                    }
                    State { state }
                })
                .collect()
        }
    }

    pub fn default_config() -> GameConfig {
        GameConfig {
            sort_kinds: vec!["A", "B", "X", "Y", "Z"]
                .into_iter()
                .map(|str| Sort(str.to_string()))
                .collect(),
            cards_number: 6,
            cards_sort: vec![
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
            player_number: 3,
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PlCard {
        pub hand: Card,
        pub head: Card,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct State {
        state: Vec<PlCard>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct View {
        pub hand: Card,
        pub other: Vec<(Player, Card)>,
    }

    impl State {
        pub fn rand<T>(config: &GameConfig, rng: &mut T) -> Self
        where
            T: rand::Rng,
        {
            let all_states = config.all_states();
            let i = rng.gen_range(0..all_states.len());
            all_states.into_iter().nth(i).unwrap()
        }

        pub fn players_head(&self, player: Player) -> Card {
            self.state[player.0].head
        }

        pub fn players_hand(&self, player: Player) -> Card {
            self.state[player.0].hand
        }

        pub fn cards_from_player(&self, player: Player) -> View {
            let hand = self.players_hand(player);
            let other = self
                .state
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    if i != player.0 {
                        Some((Player(i), c.head))
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
        Declare { declare: Vec<(Sort, bool)> }, // 全てのソートについて回答している必要がある。
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Ans {
        QueAns(usize),
        DecAns(bool),
    }

    // 妥当でない move には None を返す。
    // query を入れると queans が返る
    // declare を入れると decans が返る。
    pub fn answer(
        config: &GameConfig,
        state: &State,
        player: Player,
        move_player: &Move,
    ) -> Option<Ans> {
        match move_player {
            Move::Query {
                query_to,
                query_sort,
            } => {
                if *query_to == player {
                    return None;
                }
                let View { hand, other } = state.cards_from_player(*query_to);
                let sort_num: usize = other
                    .into_iter()
                    .map(|(_, c)| c)
                    .chain(std::iter::once(hand))
                    .filter(|c| config.has_sort(c, query_sort))
                    .count();
                Some(Ans::QueAns(sort_num))
            }
            Move::Declare { declare } => {
                let head = state.players_head(player);
                let mut sort_set = HashSet::new();
                for (s, b) in declare {
                    if config.has_sort(&head, s) != *b {
                        return Some(Ans::DecAns(false));
                    }
                    sort_set.insert(s.clone());
                }
                // 全ての sort について宣言する必要がある。
                if sort_set != config.all_sort().into_iter().collect::<HashSet<_>>() {
                    None
                } else {
                    Some(Ans::DecAns(true))
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Game {
        config: GameConfig,
        state: State,
        history: Vec<(Move, Option<Ans>)>,
        turn: usize,
        remain: HashSet<Player>,
        win: Option<Player>,
    }

    impl Game {
        pub fn config(&self) -> &GameConfig {
            &self.config
        }
        pub fn gen_from_state(config: &GameConfig, state: State) -> Self {
            Self {
                config: config.clone(),
                state,
                history: vec![],
                turn: 0,
                remain: config.all_player().into_iter().collect(),
                win: None,
            }
        }

        pub fn turn(&self) -> Player {
            self.config.player_turn(self.turn)
        }

        pub fn view_from_player(&self, player: Player) -> View {
            self.state.cards_from_player(player)
        }

        pub fn history(&self) -> Vec<(Move, Option<Ans>)> {
            self.history.clone()
        }

        pub fn is_win(&self) -> Option<Player> {
            self.win
        }

        pub fn move_game(&mut self, m: Move) -> Option<Ans> {
            // 誰かが勝っているのでゲームは進まない
            if self.win.is_some() {
                return None;
            }

            // とりあえず move に対する回答を出す。
            // ただし、ゲームの状況次第で ans をなかったことにする。
            let mut ans = answer(&self.config, &self.state, self.turn(), &m);

            // このプレイヤーがゲームに勝ったか負けたかを表す。
            let mut win: Option<bool> = None;

            // すでに負けていてゲームで適当な行動ができない。
            if !self.remain.contains(&self.turn()) {
                (win, ans) = (Some(false), None);
            }

            // 同じ手をとった場合反則負け
            let movable = self
                .history
                .iter()
                .skip(self.turn().0)
                .step_by(3)
                .all(|h| h.0 != m);
            if win.is_none() && !movable {
                (win, ans) = (Some(false), None);
            }

            match ans {
                // やってはいけない行動をしたので負け
                None => {
                    (win, ans) = (Some(false), None); // やってはいけないなので、回答も None にする
                }
                // 宣言が間違っていたので負け
                Some(Ans::DecAns(false)) => {
                    win = Some(false);
                }
                // 宣言があっていたので勝ち
                Some(Ans::DecAns(true)) => {
                    win = Some(true);
                }
                _ => {}
            }

            match win {
                // 負けた場合
                Some(false) => {
                    self.remain.remove(&self.turn());
                    // 残り一人になったらその人を勝ちにする。
                    if self.remain.len() == 1 {
                        self.win = Some(*(self.remain.iter().next().unwrap()));
                    }
                }
                // 勝った場合
                Some(true) => {
                    self.win = Some(self.turn());
                }
                // 何もなかった場合
                None => {}
            }

            // 勝ち負けによらずに、ターンを増やして ans を使う。
            self.turn += 1;
            self.history.push((m, ans.clone()));
            ans
        }
    }
}

pub mod utils {
    use super::game::*;
    use itertools::iproduct;
    use std::collections::HashSet;

    pub fn all_query(config: &GameConfig) -> impl Iterator<Item = Move> {
        iproduct!(config.all_player(), config.all_sort()).map(|(player_num, sort)| Move::Query {
            query_to: player_num,
            query_sort: sort,
        })
    }

    pub fn possible_states(
        config: GameConfig,
        player: Player,
        view: View,
        history: Vec<(Move, Option<Ans>)>,
    ) -> impl Iterator<Item = State> {
        config.all_states().into_iter().filter(move |state| {
            let state_view = state.cards_from_player(player);
            view == state_view
                && history
                    .iter()
                    .enumerate()
                    .all(|(turn, (history_move, history_ans))| {
                        let ans_state =
                            answer(&config, state, config.player_turn(turn), history_move);
                        ans_state == *history_ans
                    })
        })
    }

    pub fn answerable(
        config: GameConfig,
        player: Player,
        view: View,
        history: Vec<(Move, Option<Ans>)>,
    ) -> Option<Vec<(Sort, bool)>> {
        let possible_head: Vec<Card> = possible_states(config.clone(), player, view, history)
            .map(|state| state.players_head(player))
            .collect();

        config
            .all_sort()
            .into_iter()
            .map(|sort| {
                let b: HashSet<bool> = possible_head
                    .iter()
                    .map(|head| config.has_sort(head, &sort))
                    .collect();
                if b.len() == 1 {
                    Some((sort, b.into_iter().next().unwrap()))
                } else {
                    None
                }
            })
            .collect::<Option<_>>()
    }
}

pub mod agent;

#[cfg(test)]
mod tests {
    use super::agent::{Agent, RandomPlayer, UseEntropyPlayer};
    use super::game::{default_config, Game};
    use rand::Rng;

    #[test]
    fn test_randoms() {
        let config = default_config();

        let state = {
            let all_states = config.all_states();
            let mut thread_rand = rand::thread_rng();
            let i = thread_rand.gen_range(0..all_states.len());
            all_states.into_iter().nth(i).unwrap()
        };

        let mut game = Game::gen_from_state(&config, state);
        let agent0 = RandomPlayer::default();
        let agent1 = RandomPlayer::default();
        let agent2 = RandomPlayer::default();

        let mut players: Vec<Box<dyn Agent>> =
            vec![Box::new(agent0), Box::new(agent1), Box::new(agent2)];

        loop {
            let player = game.turn();
            let agent = &mut players[player.0];
            let player_move = agent.which(
                config.clone(),
                player,
                game.view_from_player(player),
                game.history(),
            );
            let ans = game.move_game(player_move.clone());
            println!("player:{player:?} move: {player_move:?} answer: {ans:?}");

            if let Some(win) = game.is_win() {
                println!("win: {win:?}");
                break;
            }
        }
    }

    #[test]
    fn test_entropy() {
        let config = default_config();

        let state = {
            let all_states = config.all_states();
            let mut thread_rand = rand::thread_rng();
            let i = thread_rand.gen_range(0..all_states.len());
            all_states.into_iter().nth(i).unwrap()
        };

        let mut game = Game::gen_from_state(&config, state);
        let agent0 = UseEntropyPlayer;
        let agent1 = UseEntropyPlayer;
        let agent2 = UseEntropyPlayer;

        let mut players: Vec<Box<dyn Agent>> =
            vec![Box::new(agent0), Box::new(agent1), Box::new(agent2)];

        loop {
            let player = game.turn();
            let agent = &mut players[player.0];
            let player_move = agent.which(
                config.clone(),
                player,
                game.view_from_player(player),
                game.history(),
            );
            let ans = game.move_game(player_move.clone());
            println!("player:{player:?} move: {player_move:?} answer: {ans:?}");

            if let Some(win) = game.is_win() {
                println!("win: {win:?}");
                break;
            }
        }
    }
}
