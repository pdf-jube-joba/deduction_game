use game::abstract_game::*;
use rand::thread_rng;

fn main() {
    #[cfg(target_arch = "x86_64")]
    {
        let config = game::utils::default_config();
        let mut game = config.gen_random(&mut thread_rng());
        let user = game::agent::CUIUser;
        let agent1 = game::agent::RandomPlayer::default();
        let agent2 = game::agent::RandomPlayer::default();

        let mut players: Vec<Box<dyn Agent<Game = game::defs::Game>>> =
            vec![Box::new(user), Box::new(agent1), Box::new(agent2)];

        loop {
            let player = game.player_turn();
            let agent = &mut players[player];
            let info = game.info_and_move_now();
            let m = agent.use_info(info.0, info.1);
            if !game.move_game(m.clone()) {
                panic!("有効な手でなかった")
            }

            if let Some(win) = game.is_win() {
                println!("win: {win:?}");
                break;
            }
        }
    }
}
