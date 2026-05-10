pub mod agent;

pub use agent::*;

#[cfg(test)]
mod tests {
    use crate::{RandomPlayer, UseEntropyPlayer};
    use game_core::abstract_game::*;
    use game_core::defs::{Game, GameConfig};
    use game_core::utils::default_config;
    use rand::random;

    fn test_player_with_config(
        config: GameConfig,
        mut players: Vec<Box<dyn Agent<Game = Game>>>,
    ) -> usize {
        let mut game = GameConfig::gen_random(&config, random());
        assert_eq!(players.len(), config.player_num());
        eprintln!("{game:?}");
        let mut i = 0;
        while game.is_win().is_none() {
            let player = game.player_turn();
            let agent = &mut players[player];
            let info = game.info_and_move_now();
            let m = agent.use_info(info.0, info.1);
            eprintln!("{player:?} {m:?}");
            i += 1;
            if !game.move_game(m) {
                panic!("有効でない move を返した！")
            }
        }
        i
    }

    #[test]
    fn test_randoms() {
        let config = default_config();
        let agent0 = RandomPlayer::default();
        let agent1 = RandomPlayer::default();
        let agent2 = RandomPlayer::default();
        let players: Vec<Box<dyn Agent<Game = Game>>> =
            vec![Box::new(agent0), Box::new(agent1), Box::new(agent2)];
        test_player_with_config(config, players);
    }

    #[test]
    fn test_entropy() {
        let config = default_config();
        let agent0 = UseEntropyPlayer;
        let agent1 = UseEntropyPlayer;
        let agent2 = UseEntropyPlayer;
        let players: Vec<Box<dyn Agent<Game = Game>>> =
            vec![Box::new(agent0), Box::new(agent1), Box::new(agent2)];
        test_player_with_config(config, players);
    }

    #[test]
    #[ignore = "search-style test that can take a long time to find a 4-move game"]
    fn find4() {
        let config = default_config();

        let agent0 = RandomPlayer::default();
        let agent1 = RandomPlayer::default();
        let agent2 = RandomPlayer::default();
        let mut t = 0;
        loop {
            let players: Vec<Box<dyn Agent<Game = Game>>> = vec![
                Box::new(agent0.clone()),
                Box::new(agent1.clone()),
                Box::new(agent2.clone()),
            ];
            t += 1;
            let i = test_player_with_config(config.clone(), players);
            if i == 4 {
                break;
            }
            print!(".");
        }
        print!("{t}");
    }
}
