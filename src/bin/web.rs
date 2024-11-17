use game::{
    abstract_game::{Agent, ImperfectInfoGame, Player},
    agent::*,
    defs::*,
    utils::default_config,
};
use gloo::timers::callback::Interval;
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use std::collections::HashSet;
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
                    declare: declare.iter().cloned().collect(),
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum WebOpponent {
    Random,
    Entropy,
}

fn all_strategy() -> Vec<WebOpponent> {
    vec![WebOpponent::Random, WebOpponent::Entropy]
}

fn map_opp(m: WebOpponent) -> Opponent {
    match m {
        WebOpponent::Random => {
            Opponent::RandomSmallRng(RandomPlayer::new(SmallRng::from_entropy()))
        }
        WebOpponent::Entropy => Opponent::Entoropy(UseEntropyPlayer::default()),
    }
}

fn map_strategy_name(m: WebOpponent) -> String {
    match m {
        WebOpponent::Random => "Random".to_string(),
        WebOpponent::Entropy => "Entropy".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PlaySetting {
    as_player: Player,
    opponent_strategy: Vec<Option<WebOpponent>>,
}

impl PlaySetting {
    fn new(config: &GameConfig) -> Self {
        let mut opponent_strategy = vec![None; config.player_num()];
        for i in 1..config.player_num() {
            opponent_strategy[i] = Some(WebOpponent::Random);
        }
        PlaySetting {
            as_player: 0,
            opponent_strategy,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SettingScene {
    play_setting: PlaySetting,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct SettingSceneProps {
    config: GameConfig,
    change_gameconfig: Callback<GameConfig>,
    setting_end: Callback<PlaySetting>,
}

#[derive(Debug, Clone, PartialEq)]
enum SettingSceneMsg {
    ChangeStrategy(usize, WebOpponent),
    PlayAsThis(usize),
    OnEnd,
}

impl Component for SettingScene {
    type Message = SettingSceneMsg;
    type Properties = SettingSceneProps;
    fn create(ctx: &Context<Self>) -> Self {
        let SettingSceneProps {
            config,
            change_gameconfig,
            setting_end,
        } = ctx.props();
        Self {
            play_setting: PlaySetting::new(config),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let SettingSceneProps {
            config,
            change_gameconfig,
            setting_end,
        } = ctx.props().clone();
        let mut h: Vec<Html> = vec![];
        for i in 0..config.player_num() {
            if i == self.play_setting.as_player {
                h.push(html! { {"player"} })
            } else {
                h.push(html! { {format!("p({i})")} });
                for m in all_strategy() {
                    let onclick = ctx
                        .link()
                        .callback(move |_: MouseEvent| SettingSceneMsg::ChangeStrategy(i, m));
                    let b = if Some(m) == self.play_setting.opponent_strategy[i] {
                        "t"
                    } else {
                        "f"
                    };
                    h.push(html! {
                        <button onclick={onclick}> {map_strategy_name(m)} {format!("--{b}")} </button>
                    });
                }
                let onclick = ctx
                    .link()
                    .callback(move |_: MouseEvent| SettingSceneMsg::PlayAsThis(i));
                h.push(html! { <button onclick={onclick}> {"play as this turn"} </button> })
            }
            h.push(html! {<br/>})
        }

        let onclick = ctx.link().callback(|_: MouseEvent| SettingSceneMsg::OnEnd);
        h.push(html! {<button onclick={onclick}> {"start"} </button>});
        html! { {for h} }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SettingSceneMsg::ChangeStrategy(i, s) => {
                self.play_setting.opponent_strategy[i] = Some(s);
                true
            }
            SettingSceneMsg::OnEnd => {
                ctx.props().setting_end.emit(self.play_setting.clone());
                true
            }
            SettingSceneMsg::PlayAsThis(i) => {
                let p = self.play_setting.as_player;
                self.play_setting.opponent_strategy.swap(i, p);
                self.play_setting.as_player = i;
                true
            }
        }
    }
}

struct GameScene {
    game: Game,
    as_player: usize,
    other_players: Vec<Option<Opponent>>,
    #[allow(unused)]
    interval: Interval,
}

#[derive(Debug, Clone, PartialEq)]
enum GameSceneMsg {
    Move(Move),
    OtherMove,
}

#[derive(Clone, PartialEq, Properties)]
struct GameSceneProps {
    config: GameConfig,
    play_setting: PlaySetting,
}

impl GameSceneProps {
    fn new(config: GameConfig, play_setting: PlaySetting) -> Option<Self> {
        let PlaySetting {
            as_player,
            opponent_strategy,
        } = &play_setting;
        if config.player_num() != opponent_strategy.len() {
            return None;
        }
        for (i, v) in opponent_strategy.iter().enumerate() {
            if i == *as_player && opponent_strategy[i].is_some() {
                return None;
            }
            if i != *as_player && opponent_strategy[i].is_none() {
                return None;
            }
        }
        Some(Self {
            config,
            play_setting,
        })
    }
}

impl Component for GameScene {
    type Message = GameSceneMsg;
    type Properties = GameSceneProps;
    fn create(ctx: &Context<Self>) -> Self {
        let GameSceneProps {
            config,
            play_setting,
        } = ctx.props();
        let callback = ctx.link().callback(|_| GameSceneMsg::OtherMove);
        let other_players = play_setting
            .opponent_strategy
            .iter()
            .map(|o| o.as_ref().map(|o| map_opp(*o)))
            .collect();
        Self {
            game: config.gen_random(&mut thread_rng()),
            as_player: play_setting.as_player,
            other_players,
            interval: Interval::new(1000, move || callback.emit(())),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let GameSceneProps {
            config,
            play_setting: _,
        } = ctx.props();
        let move_callback = ctx.link().callback(GameSceneMsg::Move);
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

        let as_player = self.as_player;

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
        let as_player = self.as_player;
        let who_turn = self.game.player_turn();
        log(format!("{msg:?} {who_turn} {as_player}"));
        match msg {
            GameSceneMsg::Move(m) if who_turn == as_player => {
                let b = self.game.move_game(m);
                if !b {
                    log("有効でない")
                }
            }
            GameSceneMsg::OtherMove if who_turn != as_player => {
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

enum Scene {
    Setting,
    Game,
}

struct App {
    config: GameConfig,
    play_setting: PlaySetting,
    scene: Scene,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct Props {}

enum Msg {
    SettingChange(GameConfig),
    GameStart(PlaySetting),
}

impl Component for App {
    type Message = Msg;
    type Properties = Props;
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            config: default_config(),
            scene: Scene::Setting,
            play_setting: PlaySetting::new(&default_config()),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.scene {
            Scene::Setting => {
                let change_gameconfig = ctx
                    .link()
                    .callback(|config: GameConfig| Msg::SettingChange(config));
                let setting_end = ctx.link().callback(Msg::GameStart);
                html! {
                    <SettingScene config={self.config.clone()} change_gameconfig={change_gameconfig} setting_end={setting_end}/>
                }
            }
            Scene::Game => {
                html! {
                    <GameScene config={self.config.clone()} play_setting={self.play_setting.clone()} />
                }
            }
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SettingChange(game_config) => {
                self.config = game_config;
                true
            }
            Msg::GameStart(p) => {
                self.play_setting = p;
                self.scene = Scene::Game;
                true
            }
        }
    }
}

fn main() {
    let document = gloo::utils::document();
    let target_element = document.get_element_by_id("main").unwrap();
    yew::Renderer::<App>::with_root_and_props(target_element, Props {}).render();
}
