///  Player(i) がいるなら j < i に対して Player(j) もいること
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
