use std::collections::HashSet;

use game::{
    abstract_game::{Agent, ImperfectInfoGame, Player},
    agent::*,
    defs::*,
    utils::default_config,
};
use gloo::timers::callback::Interval;
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use yew::prelude::*;

pub fn log<S>(s: S)
where
    S: AsRef<str>,
{
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(s.as_ref()))
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
        .iter()
        .fold(String::new(), |s, s1| format!("{s} {s1}"));
    html! {
        format!("({})", s)
    }
}

struct MoveView {
    declare: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct MoveProps {
    as_player: Player,
    config: GameConfig,
    callback: Callback<Move>,
}

enum MoveMsg {
    Toggle(usize),
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
            declare: vec![false; config.cards_num()],
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let MoveProps {
            as_player,
            config,
            callback,
        } = ctx.props().clone();
        // query to other player
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

        // declare cards
        let declare_html = {
            let mut declare_html = vec![];
            for i in 0..config.cards_num() {
                let callback = ctx.link().callback(move |_: MouseEvent| MoveMsg::Toggle(i));
                let card_select = html! {
                    <button onclick={callback}> {if self.declare[i] {"t"} else {"f"}} </button>
                };
                declare_html.push(card_select);
            }

            let declare: HashSet<_> = self
                .declare
                .iter()
                .enumerate()
                .filter_map(|(i, b)| if *b { Some(Card(i)) } else { None })
                .collect();
            let onclick = Callback::from(move |_: MouseEvent| {
                callback.emit(Move::Declare {
                    declare: declare.clone(),
                })
            });
            declare_html
                .push(html! {<> <button onclick={onclick}> {"declare"} </button> <br/> </>});
            declare_html
        };

        html! {
            <>
                {for other_player_htmls.into_iter().flatten() }
                {declare_html}
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MoveMsg::Toggle(i) => {
                self.declare[i] = !self.declare[i];
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
                    {format!(" A: {ans}")} <br/>
                </>},
                MoveAns::Declare { declare, ans } => html! {<>
                    {format!("Q: {declare:?} ")} <br/>
                    {format!(" A: {ans}")} <br/>
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
            h.push(html! {<CardView config={config.clone()} card={*c}/>});
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
    other_players: Vec<Option<Opponent<rand::rngs::SmallRng>>>,
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
    players: Vec<Option<Opponent<rand::rngs::SmallRng>>>,
}

impl GameProps {
    fn new(
        config: GameConfig,
        as_player: Player,
        players: Vec<Option<Opponent<rand::rngs::SmallRng>>>,
    ) -> Option<Self> {
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
        log(format!("{msg:?} {who_turn} {as_player}"));
        match msg {
            GameMsg::Move(m) if who_turn == *as_player => {
                let b = self.game.move_game(m);
                if !b {
                    log("有効でない")
                }
            }
            GameMsg::OtherMove if who_turn != *as_player => {
                let p = self.other_players[who_turn].as_mut().unwrap();
                let info = self.game.info_and_move_now();
                let m = p.use_info(info.0, info.1);
                self.game.move_game(m);
            }
            _ => {}
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
