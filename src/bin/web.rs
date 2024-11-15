use rand::thread_rng;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq, Properties)]
struct CardViewProps {
    config: game::game::GameConfig,
    card: game::game::Card,
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
    declare: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct MoveProps {
    all_sorts: Vec<game::game::Sort>,
    callback: Callback<game::game::Move>,
}

enum MoveMsg {
    Toggle(usize),
}

impl Component for MoveView {
    type Message = MoveMsg;
    type Properties = MoveProps;
    fn create(ctx: &Context<Self>) -> Self {
        let MoveProps {
            all_sorts,
            callback: _,
        } = ctx.props();
        Self {
            declare: vec![false; all_sorts.len()],
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let MoveProps {
            all_sorts,
            callback,
        } = ctx.props().clone();
        let mut player1_query_htmls = vec![];
        for s in &all_sorts {
            let snew = s.clone();
            let callback = callback.clone();
            let onclick = Callback::from(move |_: MouseEvent| {
                callback.emit(game::game::Move::Query {
                    query_to: game::game::Player(1),
                    query_sort: snew.clone(),
                })
            });
            let html = html! {
                <button class="button" onclick={onclick}> {s.to_string()} </button>
            };
            player1_query_htmls.push(html);
        }

        let mut player2_query_htmls = vec![];
        for s in &all_sorts {
            let snew = s.clone();
            let callback = callback.clone();
            let onclick = Callback::from(move |_: MouseEvent| {
                callback.emit(game::game::Move::Query {
                    query_to: game::game::Player(2),
                    query_sort: snew.clone(),
                })
            });
            let html = html! {
                <button class="button" onclick={onclick}> {s.to_string()} </button>
            };
            player2_query_htmls.push(html);
        }

        let mut declare_html = vec![];
        for (i, s) in all_sorts.iter().enumerate() {
            let snew = s.clone();
            let callback = ctx.link().callback(move |_: MouseEvent| MoveMsg::Toggle(i));
            let h = html! {
                <button onclick={callback}> {format!("{snew}: {}", if self.declare[i] {"t"} else {"f"})} </button>
            };
            declare_html.push(h);
        }

        let declare: Vec<_> = all_sorts
            .iter()
            .enumerate()
            .map(|(i, s)| (s.clone(), self.declare[i]))
            .collect();

        html! {
            <>
                {"query to 1" } {for player1_query_htmls} <br/>
                {"query to 2 " } {for player2_query_htmls} <br/>
                <button onclick={Callback::from(move |_: MouseEvent|{
                    callback.emit(game::game::Move::Declare { declare: declare.clone() })
                })}> {"declare"} </button> {for declare_html} <br/>
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
    history: Vec<(game::game::Move, Option<game::game::Ans>)>,
}

#[function_component(HistoryView)]
fn history_view(HistoryProps { history }: &HistoryProps) -> Html {
    history
        .iter()
        .map(|(m, a)| {
            let m = match m {
                game::game::Move::Query {
                    query_to,
                    query_sort,
                } => {
                    format!("Q: {query_to:?} {query_sort:?}")
                }
                game::game::Move::Declare { declare } => {
                    format!("Q: {declare:?}")
                }
            };

            let a = match a {
                None => "A: None".to_string(),
                Some(game::game::Ans::QueAns(n)) => {
                    format!("A: {n}")
                }
                Some(game::game::Ans::DecAns(b)) => {
                    format!("A: {b}")
                }
            };

            html! {
                <>
                    {m} <br/>
                    {a} <br/>
                </>
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct PlayerViewProps {
    config: game::game::GameConfig,
    view: game::game::View,
}

#[function_component(PlayerView)]
fn player_view(PlayerViewProps { config, view }: &PlayerViewProps) -> Html {
    let other = view.other.iter().map(|(p, c)| {
        html! {
            <>
            {format!("p({:?}) ", p.0)}
            <CardView config={config.clone()} card={*c}/> <br/>
            </>
        }
    });
    html! {
        <>
            {"hand "} <CardView config={config.clone()} card={view.hand}/> <br/>
            {for other}
        </>
    }
}

struct App {
    game: game::game::Game,
    p1: Box<dyn game::agent::Agent>,
    p2: Box<dyn game::agent::Agent>,
}

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Move(game::game::Move),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        let mut rng = thread_rng();
        let config = game::game::default_config();
        let state = game::game::State::rand(&config, &mut rng);
        let game = game::game::Game::gen_from_state(&config, state);
        let p1 = game::agent::RandomPlayer::default();
        let p2 = game::agent::RandomPlayer::default();
        Self {
            game,
            p1: Box::new(p1),
            p2: Box::new(p2),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let callback = ctx.link().callback(Msg::Move);
        let win = if let Some(p) = self.game.is_win() {
            html! {{format!("win: {}", p.0)}}
        } else {
            html! {{"game goes"}}
        };
        let all_card = self.game.config().all_cards().into_iter().map(|c| {
            html! {
                <>
                <CardView config={self.game.config().clone()} card={c}/> {" "}
                </>
            }
        });

        html! {
            <>
            {for all_card} <br/>
            {win} <br/>
            <PlayerView config={self.game.config().clone()} view={self.game.view_from_player(game::game::Player(0))}/>
            <MoveView all_sorts={self.game.config().all_sort()} callback={callback}/>
            <HistoryView history={self.game.history()}/>
            </>
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Move(m) => {
                self.game.move_game(m);
                self.p1.game(&mut self.game);
                self.p2.game(&mut self.game);
            }
        }
        true
    }
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("main").unwrap();
    yew::Renderer::<App>::with_root(target_element).render();
}
