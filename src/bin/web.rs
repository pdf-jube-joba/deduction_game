use game::{
    abstract_game::{Agent, ImperfectInfoGame, Player},
    agent::*,
    defs::*,
    utils::default_config,
};
use gloo::timers::callback::Interval;
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Opponent {
    Random(RandomPlayer<rand::rngs::SmallRng>),
    Entoropy(UseEntropyPlayer),
}

impl Agent for Opponent {
    type Game = Game;
    fn use_info(
        &mut self,
        info: <Self::Game as ImperfectInfoGame>::Info,
        possible_moves: Vec<<Self::Game as ImperfectInfoGame>::Move>,
    ) -> <Self::Game as ImperfectInfoGame>::Move {
        match self {
            Opponent::Random(p) => p.use_info(info, possible_moves),
            Opponent::Entoropy(p) => p.use_info(info, possible_moves),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct CardViewProps {
    config: GameConfig,
    card: Card,
}

#[function_component(CardView)]
fn card_view(CardViewProps { config, card }: &CardViewProps) -> Html {
    let s: String = config
        .all_sort_of_card(card)
        .into_iter()
        .fold(String::new(), |s, s1| format!("{s} {s1}"));
    html! {
        format!("({})", s)
    }
}

struct MoveView {
    declare: Vec<Vec<bool>>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct MoveProps {
    as_player: Player,
    config: GameConfig,
    callback: Callback<Move>,
}

enum MoveMsg {
    Toggle(usize, usize),
}

impl Component for MoveView {
    type Message = MoveMsg;
    type Properties = MoveProps;
    fn create(ctx: &Context<Self>) -> Self {
        let MoveProps {
            as_player: _,
            config,
            callback: _,
        } = ctx.props();
        Self {
            declare: vec![vec![false; config.all_sort().len()]; config.head_num() - 1],
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let MoveProps {
            as_player,
            config,
            callback,
        } = ctx.props().clone();
        let mut other_player_htmls = vec![];
        for i in 0..config.player_num() {
            if i != as_player {
                let mut htmls = vec![html! {format!("query to {i}")}];
                for s in config.all_sort() {
                    let snew = s.clone();
                    let callback = callback.clone();
                    let onclick = Callback::from(move |_: MouseEvent| {
                        callback.emit(Move::Query {
                            query_to: i,
                            query_sort: snew.clone(),
                        })
                    });
                    htmls.push(html! {
                        <button class="button" onclick={onclick}> {s.to_string()} </button>
                    });
                }
                htmls.push(html! {<br/>});
                other_player_htmls.push(htmls);
            };
        }

        let mut deduction_html = vec![];
        for i in 0..config.player_num() {
            if i != as_player {
                let mut htmls = vec![];
                for (j, s) in config.all_sort().into_iter().enumerate() {
                    let snew = s.clone();
                    let callback = ctx
                        .link()
                        .callback(move |_: MouseEvent| MoveMsg::Toggle(i, j));
                    htmls.push(html! {
                        <button onclick={callback}> {format!("{snew}: {}", if self.declare[i][j] {"t"} else {"f"})} </button>
                    });
                }
                htmls.push(html! {<br/>});
                deduction_html.push(htmls);
            }
        }

        let declare_html = {
            let mut declare = vec![];
            for i in 0..config.head_num() {
                declare.push(
                    config
                        .all_sort()
                        .into_iter()
                        .enumerate()
                        .filter_map(|(j, s)| if self.declare[i][j] { Some(s) } else { None })
                        .collect(),
                );
            }
            html! {<button onclick={Callback::from(move |_: MouseEvent| {
                callback.emit(Move::Declare { declare: declare.clone() });
            })}> {"declare"} </button>}
        };

        html! {
            <>
                {for other_player_htmls.into_iter().flatten() }
                {for deduction_html.into_iter().flatten()}
                {declare_html}
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MoveMsg::Toggle(i, j) => {
                self.declare[i][j] = !self.declare[i][j];
                true
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct HistoryProps {
    history: Vec<MoveAns>,
}

#[function_component(HistoryView)]
fn history_view(HistoryProps { history }: &HistoryProps) -> Html {
    history
        .iter()
        .map(|qa| {
            let h: Html = match qa {
                MoveAns::Query {
                    query_to,
                    query_sort,
                    ans,
                } => html! {<>
                    {format!("Q: {query_to} {query_sort}")} <br/>
                    {format!(" A: {ans}")}
                </>},
                MoveAns::Declare { declare, ans } => html! {<>
                    {format!("Q: {declare:?} ")} <br/>
                    {format!(" A: {ans}")}
                </>},
            };

            html! {h}
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct PlayerViewProps {
    config: GameConfig,
    view: View,
}

#[function_component(PlayerView)]
fn player_view(PlayerViewProps { config, view }: &PlayerViewProps) -> Html {
    let hand: Vec<Html> = {
        let mut h: Vec<_> = vec![html! {"hand: "}];
        for c in &view.hand {
            h.push(html! {<CardView config={config.clone()} card={c.clone()}/>});
        }
        h.push(html! {<br/>});
        h
    };

    let other = view.other.iter().enumerate().filter_map(|(p, cs)| {
        let cs = cs.as_ref()?;
        Some(html! {
            <>
            {format!("p({:?}) ", p)}
            {for cs.iter().map(|c| html!{<CardView config={config.clone()} card={*c}/>})} <br/>
            </>
        })
    });

    html! {
        <>
            {hand}
            {for other}
        </>
    }
}

struct GameApp {
    game: Game,
    other_players: Vec<Option<Opponent>>,
    movable: bool,
    interval: Interval,
}

#[derive(Debug, Clone, PartialEq)]
enum GameMsg {
    Move(Move),
    OtherMove,
}

#[derive(Clone, PartialEq, Properties)]
struct GameProps {
    config: GameConfig,
    as_player: Player,
    players: Vec<Option<Opponent>>,
}

impl GameProps {
    fn new(config: GameConfig, as_player: Player, players: Vec<Option<Opponent>>) -> Option<Self> {
        if config.player_num() != players.len() {
            return None;
        }
        for (i, v) in players.iter().enumerate() {
            if i == as_player && players[i].is_some() {
                return None;
            }
            if i != as_player && players[i].is_none() {
                return None;
            }
        }
        Some(Self {
            config,
            as_player,
            players,
        })
    }
}

impl Component for GameApp {
    type Message = GameMsg;
    type Properties = GameProps;
    fn create(ctx: &Context<Self>) -> Self {
        let GameProps {
            config,
            players,
            as_player,
        } = ctx.props();
        let callback = ctx.link().callback(|_| GameMsg::OtherMove);
        Self {
            game: config.gen_random(&mut thread_rng()),
            other_players: players.clone(),
            movable: *as_player == 0,
            interval: Interval::new(1000, move || callback.emit(())),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let GameProps {
            config,
            players: _,
            as_player,
        } = ctx.props();
        let move_callback = ctx.link().callback(GameMsg::Move);
        let win = if let Some(p) = self.game.is_win() {
            html! {{format!("win: {p:?}")}}
        } else {
            html! {{"game goes"}}
        };
        let all_card = config.all_cards().into_iter().map(|c| {
            html! {
                <>
                <CardView config={config.clone()} card={c}/> {" "}
                </>
            }
        });

        let Info {
            config,
            query_answer,
            view,
        } = self.game.info_and_move_now().0;

        html! {
            <>
            {"cards:"} {for all_card} <br/>
            {win} <br/>
            <PlayerView config={config.clone()} view={view}/>
            <MoveView as_player={as_player} config={config.clone()} callback={move_callback}/>
            <HistoryView history={query_answer}/>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let GameProps {
            config: _,
            as_player,
            players: _,
        } = ctx.props();
        let who_turn = self.game.player_turn();
        match msg {
            GameMsg::Move(m) if who_turn == *as_player => {
                self.game.move_game(m);
            }
            GameMsg::OtherMove if who_turn != *as_player => {
                let p = self.other_players[who_turn].as_mut().unwrap();
                let info = self.game.info_and_move_now();
                let m = p.use_info(info.0, info.1);
                self.game.move_game(m);
            }
            _ => {
                return false;
            }
        }
        true
    }
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("main").unwrap();
    let props = GameProps::new(
        default_config(),
        0,
        vec![
            None,
            Some(Opponent::Random(
                RandomPlayer::new(SmallRng::from_entropy()),
            )),
            Some(Opponent::Random(
                RandomPlayer::new(SmallRng::from_entropy()),
            )),
        ],
    )
    .unwrap();
    yew::Renderer::<GameApp>::with_root_and_props(target_element, props).render();
}
