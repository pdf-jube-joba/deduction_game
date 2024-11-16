pub type Player = usize;

pub trait ImperfectInfoGame {
    type Info;
    type Move;
    // how many player there is
    fn player_number(&self) -> usize;
    // who should play
    fn player_turn(&self) -> Player;
    fn info_and_move_now(&self) -> (Self::Info, Vec<Self::Move>);
    fn move_game(&mut self, m: Self::Move) -> bool;
    fn is_win(&self) -> Option<Vec<usize>>;
}

pub trait Agent {
    type Game: ImperfectInfoGame;
    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move;
}

pub fn auto_game<G>(mut game: G, mut agents: Vec<Box<dyn Agent<Game = G>>>) -> Vec<usize>
where
    G: ImperfectInfoGame,
{
    assert_eq!(agents.len(), game.player_number());
    while game.is_win().is_none() {
        let p = game.player_turn();
        let info_move = game.info_and_move_now();
        let m = agents[p].use_info(info_move.0, info_move.1);
        if !game.move_game(m) {
            panic!("動けるやつにして");
        }
    }
    game.is_win().unwrap()
}
