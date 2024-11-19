use game::{
    abstract_game::{Agent, ImperfectInfoGame, Player},
    agent::*,
    defs::*,
    utils::default_config,
};
use gloo::timers::callback::Interval;
use itertools::Itertools;
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
        .map(|s| format!("{s}"))
        .join(" ");
    html! {
        <span class={classes!("card")}>
        {format!("{}", s)}
        </span>
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

#[derive(Debug, Clone, PartialEq, Properties)]
struct GameConfigProps {
    config: GameConfig,
}

#[function_component(GameConfigView)]
fn gameconfig_view(GameConfigProps { config }: &GameConfigProps) -> Html {
    let all_cards: Html = config
        .all_cards()
        .into_iter()
        .map(|c| {
            html! {
                html!{
                    <CardView config={config.clone()} card={c} />
                }
            }
        })
        .collect();
    html! {
        <div id="setting" class={classes!("roundbox")}>
            {"game setting"}
            {"プレイする人数："} {config.player_num()} <br/>
            {"使うカード："} {all_cards} <br/>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct GameRuleProps {
    config: GameConfig,
}

#[function_component(GameRuleView)]
fn gamerule_view(GameRuleProps { config }: &GameRuleProps) -> Html {
    html! {
        <div id="rule" class={classes!("roundbox")}>
        <div class={classes!("smallbox")}>
            {"このゲームはカード当てゲームです。あなたの頭にあるカードを当てましょう"}
        </div>
        <div class={classes!("smallbox")}>
            {"それぞれのカードは属性をそれぞれ（複数）持っています。"} <br/>
            {"使うカードと属性は下に書いてある通りです。"} <br/>
            {format!("各プレイヤーは、自分の手元と頭にそれぞれ {} 枚と {} 枚のカードを持ちます。", config.hand_num(), config.head_num())} <br/>
            {"自分の手元にあるカードは自分にしか見えません。"} <br/>
            {"自分の頭にあるカードは他のプレイヤーにしか見えません。"} <br/>
        </div>
        <div class={classes!("smallbox")}>
        {"プレイヤーは順番に次の行動をすることができます。"} <br/>
        {"1. 自分の頭にあるカードを予想して宣言する：もしあっていたらそのプレイヤーの一人勝ちです。"} <br/>
        {"2. 他のプレイヤーに、ある属性について、その属性を持っているカードが何枚見えるかを質問することができます。"} <br/>
        {"行動1. と 2. はともに、その内容はほかのすべてのプレイヤーとその結果を共有することになります。"}
        </div>
        <div class={classes!("smallbox")}>
        {"自分が過去にやった行動を 2 回することはできません。"}
        </div>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SettingScene {
    config: GameConfig,
    play_setting: PlaySetting,
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct SettingSceneProps {
    setting_end: Callback<(GameConfig, PlaySetting)>,
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
        let config = default_config();
        let play_setting = PlaySetting::new(&config);
        Self {
            config,
            play_setting,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut h: Vec<Html> = vec![];
        for i in 0..self.config.player_num() {
            if i == self.play_setting.as_player {
                h.push(html! {
                    <tr>
                    <th> {format!("あなた {}", i + 1)} </th>
                    <th> {"なし"} </th>
                    {for all_strategy().into_iter().map(|opt| html!{ <th> {"なし"} </th>})}
                    <th> {"なし"} </th>
                    </tr>
                })
            } else {
                let strategy_change_html: Vec<Html> = {
                    let mut h = vec![];
                    for m in all_strategy() {
                        let onclick = ctx
                            .link()
                            .callback(move |_: MouseEvent| SettingSceneMsg::ChangeStrategy(i, m));
                        let this_or_not_class = if Some(m) == self.play_setting.opponent_strategy[i]
                        {
                            "t"
                        } else {
                            "f"
                        };
                        h.push(html! {
                        <th>
                            <button onclick={onclick} class={this_or_not_class}> {"この戦略にする"} </button>
                        </th>
                        });
                    }
                    h
                };
                let onclick = ctx
                    .link()
                    .callback(move |_: MouseEvent| SettingSceneMsg::PlayAsThis(i));
                h.push(html! {
                    <tr>
                    <th> {format!("CPU（{}）", i + 1)} </th>
                    <th> {
                        map_strategy_name(self.play_setting.opponent_strategy[i].unwrap())
                    } </th>
                    {for strategy_change_html}
                    <th> <button onclick={onclick}> {"この手番でプレイする"} </button> </th>
                    </tr>
                })
            }
        }

        let onclick = ctx.link().callback(|_: MouseEvent| SettingSceneMsg::OnEnd);

        html! {
            <>
            <GameRuleView config={self.config.clone()}/>
            <GameConfigView config={self.config.clone()}/>
            <div class={classes!("roundbox")}>
            <table>
            <caption>
            {"play setting"}
            </caption>
            <thead >
                <tr>
                    <th> {"プレイする順番"} </th>
                    <th> {"とる戦略"} </th>
                    {for all_strategy().into_iter().map(|opt| html!{ <th> {map_strategy_name(opt)} </th>})}
                    <th> </th>
                </tr>
            </thead>
            {for h}
            </table>
            </div>
            <button onclick={onclick}> {"ゲームスタート"} </button>
            </>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SettingSceneMsg::ChangeStrategy(i, s) => {
                self.play_setting.opponent_strategy[i] = Some(s);
                true
            }
            SettingSceneMsg::OnEnd => {
                ctx.props()
                    .setting_end
                    .emit((self.config.clone(), self.play_setting.clone()));
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
    GameStart((GameConfig, PlaySetting)),
}

impl Component for App {
    type Message = Msg;
    type Properties = Props;
    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            config: default_config(),
            scene: Scene::Setting,
            play_setting: PlaySetting::new(&default_config()),
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.scene {
            Scene::Setting => {
                let setting_end = ctx.link().callback(Msg::GameStart);
                html! {
                    <SettingScene setting_end={setting_end}/>
                }
            }
            Scene::Game => {
                html! {
                    <GameScene config={self.config.clone()} play_setting={self.play_setting.clone()} />
                }
            }
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GameStart((config, play_setting)) => {
                self.config = config;
                self.play_setting = play_setting;
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
