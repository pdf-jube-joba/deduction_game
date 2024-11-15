fn main() {
    #[cfg(target_arch = "x86_64")]
    {
        use rand::Rng;
        let config = game::game::default_config();

        let state = {
            let all_states = config.all_states();
            let mut thread_rand = rand::thread_rng();
            let i = thread_rand.gen_range(0..all_states.len());
            all_states.into_iter().nth(i).unwrap()
        };

        let mut game = game::game::Game::gen_from_state(&config, state);
        let user = game::agent::User;
        let agent1 = game::agent::RandomPlayer::default();
        let agent2 = game::agent::RandomPlayer::default();

        let mut players: Vec<Box<dyn game::agent::Agent>> =
            vec![Box::new(user), Box::new(agent1), Box::new(agent2)];

        let view = game.view_from_player(game::game::Player(0));
        println!("start hand:{:?} other:{:?}", view.hand, view.other);

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
            println!("player: {player:?} move: {player_move:?} answer: {ans:?}");

            if let Some(win) = game.is_win() {
                println!("win: {win:?}");
                break;
            }
        }
    }
}
