use game::{abstract_game::Player, defs::*};
use rand::thread_rng;
use yew::prelude::*;

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
        let mut h: Vec<_> = vec![html!{"hand: "}];
        for c in view.hand {
            h.push(html!{<CardView config={config.clone()} card={c}/>});
        }
        h.push(html!{<br/>});
        h
    };

    let other = view.other.iter().map(|cs| {
        let mut h: Vec<_> = vec![];
        html! {
            <>
            {format!("p({:?}) ", p)}
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
            html! {{format!("win: {}", p)}}
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
            <PlayerView config={self.game.config().clone()} view={self.game.view_from_player(0)}/>
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
