use game::abstract_game::*;
use game::agent::{Opponent, RandomPlayer, UseEntropyPlayer};
use game::defs::GameConfig;
use game::utils::default_config;
use indicatif::ProgressBar;
use plotters::prelude::*;
use rand::thread_rng;

fn test_player_with_config(config: GameConfig, mut players: Vec<Opponent>) -> usize {
    let mut rng = thread_rng();
    let mut game = GameConfig::gen_random(&config, &mut rng);
    assert_eq!(players.len(), config.player_num());

    let mut i = 0;

    while game.is_win().is_none() {
        i += 1;
        let player: usize = game.player_turn().into();
        let agent = &mut players[player];
        let info = game.info_and_move_now();
        let m = agent.use_info(info.0, info.1);
        if !game.move_game(m) {
            panic!("有効でない move を返した！")
        }
    }

    i
}

const LOOP: usize = 500;

fn main() {
    let ps: Vec<Opponent> = vec![
        // Opponent::Random(RandomPlayer::new(SmallRng::from_entropy())),
        Opponent::RandomThreadRng(RandomPlayer::default()),
        Opponent::Entoropy(UseEntropyPlayer::default()),
    ];

    for p0 in &ps {
        for p1 in &ps {
            for p2 in &ps {
                let players = vec![p0.clone(), p1.clone(), p2.clone()];
                let mut data: Vec<usize> = vec![];
                let bar = ProgressBar::new(LOOP as u64);

                for _ in 0..LOOP {
                    let t = test_player_with_config(default_config(), players.clone());
                    data.push(t);
                    bar.inc(1);
                }

                bar.finish();

                let p = |opp: &Opponent| {
                    if matches!(opp, Opponent::RandomThreadRng(_)) {
                        "random"
                    } else {
                        "entropy"
                    }
                };
                let path = format!("plot/{}_{}_{}.png", p(p0), p(p1), p(p2));
                let root = BitMapBackend::new(&path, (640, 480)).into_drawing_area();
                root.fill(&WHITE).unwrap();

                let mut chart = ChartBuilder::on(&root)
                    .x_label_area_size(35)
                    .y_label_area_size(40)
                    .margin(5)
                    .caption("Histogram Test", ("sans-serif", 50.0))
                    .build_cartesian_2d((0u32..20u32).into_segmented(), 0u32..300u32)
                    .unwrap();

                chart
                    .configure_mesh()
                    .disable_x_mesh()
                    .bold_line_style(WHITE.mix(0.3))
                    .y_desc("Count")
                    .x_desc("Bucket")
                    .axis_desc_style(("sans-serif", 15))
                    .draw()
                    .unwrap();

                chart
                    .draw_series(
                        Histogram::vertical(&chart)
                            .style(RED.mix(0.5).filled())
                            .data(data.iter().map(|x: &usize| (*x as u32, 1))),
                    )
                    .unwrap();

                let mut win = vec![0; 3];
                for t in data {
                    win[t % 3] += 1;
                }

                eprintln!("{win:?}");

                root.present().unwrap();
            }
        }
    }
}
