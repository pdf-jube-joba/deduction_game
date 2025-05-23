use game::{
    abstract_game::{Agent, ImperfectInfoGame, Player},
    agent::*,
    defs::*,
    utils::{default_config, four_midium, three_midium},
};
use gloo::timers::callback::Interval;
use itertools::Itertools;
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use std::collections::BTreeSet;
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

#[derive(Debug, Clone, PartialEq)]
struct PlaySetting {
    as_player: Player,
    opponent_strategy: Vec<Option<WebOpponent>>,
}

impl PlaySetting {
    fn new(config: &GameConfig) -> Self {
        let mut opponent_strategy = vec![None; config.player_num()];
        for a in opponent_strategy
            .iter_mut()
            .take(config.player_num())
            .skip(1)
        {
            *a = Some(WebOpponent::Random);
        }
        PlaySetting {
            as_player: 0.into(),
            opponent_strategy,
        }
    }
    fn strategy_of_player(&self, player: Player) -> &Option<WebOpponent> {
        let player: usize = player.into();
        &self.opponent_strategy[player]
    }
    fn strategy_of_player_mut(&mut self, player: Player) -> &mut Option<WebOpponent> {
        let player: usize = player.into();
        &mut self.opponent_strategy[player]
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct PlayerRepProps {
    player: Player,
    play_setting: PlaySetting,
}

#[function_component(PlayerRepView)]
fn player_rep(
    PlayerRepProps {
        player,
        play_setting,
    }: &PlayerRepProps,
) -> Html {
    if *player == play_setting.as_player {
        html! {format!("あなた（{}）", player)}
    } else {
        html! {format!("CPU（{}）", player)}
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
        for i in config.all_player() {
            if i != as_player {
                let mut htmls = vec![html! {format!("プレイヤー{}に質問する：", i)}];
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
        let (declare_html, html2) = {
            let mut declare_html = vec![];
            let mut selected_num = 0;
            for i in 0..config.cards_num() {
                if self.declare[i] {
                    selected_num += 1;
                }
                let callback = ctx.link().callback(move |_: MouseEvent| MoveMsg::Toggle(i));
                let selected_or_not_class = if self.declare[i] {
                    classes!("selected")
                } else {
                    classes!("notselected")
                };
                let card_select = html! {
                    <button onclick={callback} class={selected_or_not_class}> {i} </button>
                };
                declare_html.push(card_select);
            }

            let declare: BTreeSet<_> = self
                .declare
                .iter()
                .enumerate()
                .filter_map(|(i, b)| if *b { Some(Card(i)) } else { None })
                .collect();

            let (selectable, onclick) = if selected_num == config.head_num() {
                (
                    classes!("declareselectable"),
                    Callback::from(move |_: MouseEvent| {
                        callback.emit(Move::Declare {
                            declare: declare.clone(),
                        })
                    }),
                )
            } else {
                (classes!("declareunselectable"), Callback::noop())
            };

            (
                declare_html,
                html! {<> <button onclick={onclick} class={selectable}> {"宣言する"} </button> </>},
            )
        };

        html! {
            <div id="moves" class={classes!("roundbox")}>
            {"あなたの行動"}
            <div>
                {for other_player_htmls.into_iter().flatten() }
            </div>
            <div>
                {"頭にあるカードを"} {html2} {"："}
                {declare_html}
            </div>
            </div>
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
    play_setting: PlaySetting,
    config: GameConfig,
}

#[function_component(HistoryView)]
fn history_view(
    HistoryProps {
        history,
        play_setting,
        config: _,
    }: &HistoryProps,
) -> Html {
    html! {
        <div class={classes!("roundbox")}>
        {"行動の履歴："} <br/>
        {for history
        .iter()
        .map(|qa| {
            let h: Html = match qa {
                MoveAns::Query {
                    who,
                    query_to,
                    query_sort,
                    ans,
                } => html! {<>
                    <PlayerRepView player={*who} play_setting={play_setting.clone()} />
                    {"から"}
                    <PlayerRepView player={*query_to} play_setting={play_setting.clone()} />
                    {"へ質問："}
                    {format!("{query_sort} は何枚ある？...{ans}")} <br/>
                </>},
                MoveAns::Declare { who, declare, ans } => html! {<>
                    <PlayerRepView player={*who} play_setting={play_setting.clone()} />
                    {"の宣言："}
                    {format!("頭のカードは {} ... {}",
                        declare.iter().map(|s| format!("{}", s.0)).join("と"),
                        if *ans {"当たり"} else {"外れ"})} <br/>
                </>},
            };

            html! {h}
        })
        }
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct PlayerViewProps {
    config: GameConfig,
    view: View,
}

#[function_component(PlayerView)]
fn player_view(PlayerViewProps { config, view }: &PlayerViewProps) -> Html {
    let hand: Vec<Html> = {
        let mut h: Vec<_> = vec![html! {"自分の手元："}];
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
            {format!("CPU({:?})の頭：", p + 1)}
            {for cs.iter().map(|c| html!{<CardView config={config.clone()} card={*c}/>})} {"、"}
            </>
        })
    });

    html! {
        <>
            <div id="view" class={classes!("roundbox")}>
            {hand}
            {for other}
            </div>
        </>
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WebOpponent {
    Random,
    Entropy,
    Search,
}

fn all_strategy() -> Vec<WebOpponent> {
    vec![
        WebOpponent::Random,
        WebOpponent::Entropy,
        WebOpponent::Search,
    ]
}

fn map_opp(m: WebOpponent) -> Opponent {
    match m {
        WebOpponent::Random => {
            Opponent::RandomSmallRng(RandomPlayer::new(SmallRng::from_entropy()))
        }
        WebOpponent::Entropy => Opponent::Entoropy(UseEntropyPlayer),
        WebOpponent::Search => Opponent::SearchPlayer(SearchPlayer::new(3)),
    }
}

fn map_strategy_name(m: WebOpponent) -> String {
    match m {
        WebOpponent::Random => "Random".to_string(),
        WebOpponent::Entropy => "Entropy".to_string(),
        WebOpponent::Search => "Search".to_string(),
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
            {"game setting"} <br/>
            {"プレイする人数："} {config.player_num()} <br/>
            {"使うカード："} {all_cards} <br/>
        </div>
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct GameRuleProps {}

#[function_component(GameRuleView)]
fn gamerule_view(GameRuleProps {}: &GameRuleProps) -> Html {
    html! {
        <div id="rule" class={classes!("roundbox")}>
        <div class={classes!("smallbox")}>
            {"このゲームはカード当てゲームです。あなたの頭にあるカードを当てましょう"}
        </div>
        <div class={classes!("smallbox")}>
            {"それぞれのカードは属性をそれぞれ（複数）持っています。使うカードと属性は下に書いてある通りです。"} <br/>
            {"各プレイヤーは、自分の手元と頭にそれぞれ何枚かのカードを持ちます。"} <br/>
            {"自分の手元にあるカードは自分にしか見えません。"} <br/>
            {"自分の頭にあるカードは他のプレイヤーにしか見えません。"} <br/>
        </div>
        <div class={classes!("smallbox")}>
        {"プレイヤーは順番に次の行動をすることができます。"} <br/>
        {"1. 自分の頭にあるカードを予想して宣言する：もしあっていたらそのプレイヤーの一人勝ちです。"} <br/>
        {"2. 他のプレイヤーに、ある属性について、その属性を持っているカードが何枚見えるかを質問することができます。"} <br/>
        {"行動1. と 2. の内容と結果は全員で共有します。"} <br/>
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum ConfigKind {
    Default,
    ThreeMidium,
    FourMidium,
}

impl ConfigKind {
    fn all() -> Vec<ConfigKind> {
        vec![
            ConfigKind::Default,
            ConfigKind::ThreeMidium,
            ConfigKind::FourMidium,
        ]
    }
    fn map_str(self) -> String {
        match self {
            ConfigKind::Default => "Default".to_string(),
            ConfigKind::ThreeMidium => "3 midium".to_string(),
            ConfigKind::FourMidium => "4 midium".to_string(),
        }
    }
}

fn map_configkind(m: ConfigKind) -> GameConfig {
    match m {
        ConfigKind::Default => default_config(),
        ConfigKind::ThreeMidium => three_midium(),
        ConfigKind::FourMidium => four_midium(),
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct SettingSceneProps {
    setting_end: Callback<(GameConfig, PlaySetting)>,
}

#[derive(Debug, Clone, PartialEq)]
enum SettingSceneMsg {
    ChangeStrategy(Player, WebOpponent),
    ChangeConfig(ConfigKind),
    PlayAsThis(Player),
    OnEnd,
}

impl Component for SettingScene {
    type Message = SettingSceneMsg;
    type Properties = SettingSceneProps;
    fn create(_ctx: &Context<Self>) -> Self {
        let config = default_config();
        let play_setting = PlaySetting::new(&config);
        Self {
            config,
            play_setting,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let choose_config: Html = ConfigKind::all()
            .into_iter()
            .map(|kind| {
                let onclick = ctx
                    .link()
                    .callback(move |_: MouseEvent| SettingSceneMsg::ChangeConfig(kind));
                html! {
                    <button onclick={onclick}> {kind.map_str()} </button>
                }
            })
            .collect();

        let mut h: Vec<Html> = vec![];
        for i in self.config.all_player() {
            if i == self.play_setting.as_player {
                h.push(html! {
                    <tr>
                    <th> <PlayerRepView player={i} play_setting={self.play_setting.clone()}/> </th>
                    <th> {"なし"} </th>
                    {for all_strategy().into_iter().map(|_| html!{ <th> {"なし"} </th>})}
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
                        let this_or_not_class =
                            if Some(m) == *self.play_setting.strategy_of_player(i) {
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
                    <th> <PlayerRepView player={i} play_setting={self.play_setting.clone()}/> </th>
                    <th> {
                        map_strategy_name(self.play_setting.strategy_of_player(i).unwrap())
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
            <GameRuleView/>
            <div id="chooseconfig" class={classes!("roundbox")}> {"ゲーム設定変更："} {choose_config}  </div>
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
                *self.play_setting.strategy_of_player_mut(i) = Some(s);
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
                let tmp = self.play_setting.strategy_of_player_mut(p).take();
                *self.play_setting.strategy_of_player_mut(p) =
                    *self.play_setting.strategy_of_player(i);
                *self.play_setting.strategy_of_player_mut(i) = tmp;
                self.play_setting.as_player = i;
                true
            }
            SettingSceneMsg::ChangeConfig(kind) => {
                let config = map_configkind(kind);
                self.play_setting = PlaySetting::new(&config);
                self.config = config.clone();
                true
            }
        }
    }
}

struct GameScene {
    game: Game,
    as_player: Player,
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
    game_end: Callback<Game>,
}

impl Component for GameScene {
    type Message = GameSceneMsg;
    type Properties = GameSceneProps;
    fn create(ctx: &Context<Self>) -> Self {
        let GameSceneProps {
            config,
            play_setting,
            game_end: _,
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
            config: _,
            play_setting,
            game_end: _,
        } = ctx.props();
        let move_callback = ctx.link().callback(GameSceneMsg::Move);

        let Info {
            config,
            query_answer,
            view: _, // この view はいまプレイしている人の view なので使ってはいけない。
        } = self.game.info_and_move_now().0;

        // player の view はこれ
        let view = self.game.view_from_player(self.as_player);

        let as_player = self.as_player;

        let who_turn = {
            let turn = self.game.player_turn();
            html! {<> <div class={classes!("roundbox")}> {"今は"} <PlayerRepView player={turn} play_setting={play_setting.clone()} /> {"の手番"} </div>  </>}
        };

        html! {
            <>
            <GameConfigView config={config.clone()}/>
            <PlayerView config={config.clone()} view={view}/>
            <MoveView as_player={as_player} config={config.clone()} callback={move_callback}/>
            {who_turn}
            <HistoryView history={query_answer} play_setting={play_setting.clone()} config={config.clone()}/>
            </>
        }
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let as_player = self.as_player;
        let who_turn = self.game.player_turn();
        log(format!("{msg:?} {who_turn} {as_player}"));
        let GameSceneProps {
            config: _,
            play_setting: _,
            game_end,
        } = ctx.props();
        match msg {
            GameSceneMsg::Move(m) if who_turn == as_player => {
                let b = self.game.move_game(m);
                if !b {
                    log("有効でない")
                }
            }
            GameSceneMsg::OtherMove if who_turn != as_player => {
                let who_turn: usize = who_turn.into();
                let p = self.other_players[who_turn].as_mut().unwrap();
                let info = self.game.info_and_move_now();
                let m = p.use_info(info.0, info.1);
                self.game.move_game(m);
            }
            _ => {}
        }
        if let Some(win) = self.game.is_win() {
            game_end.emit(self.game.clone())
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Properties)]
struct EndSceneProps {}

#[function_component(EndScene)]
fn endscene(EndSceneProps {}: &EndSceneProps) -> Html {
    // let if_win = if let Some(p) = self.game.is_win() {
    //     log(format!("{p:?}"));
    //     let p: Player = p.into_iter().position_max().unwrap().into();
    //     html! {<> {"勝者："} <PlayerRepView player={p} play_setting={play_setting.clone()}/> </> }
    // } else {
    //     html! {}
    // };
    html! {}
}

enum Scene {
    Setting,
    Game,
    End(Distr, Vec<usize>, Vec<MoveAns>),
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
    GameEnd(Game),
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
                    <GameScene config={self.config.clone()} play_setting={self.play_setting.clone()} game_end={ctx.link().callback(Msg::GameEnd)}/>
                }
            }
            Scene::End(distr, win, history) => {
                let config = self.config.clone();
                let as_player = self.play_setting.as_player;
                let play_setting = self.play_setting.clone();
                let view = distr.cards_from_player(as_player);
                let winner = {
                    let winner: Player = win
                        .iter()
                        .find_position(|point| **point != 0)
                        .unwrap()
                        .0
                        .into();
                    html! {
                        <>
                            {"勝者："} <PlayerRepView play_setting={play_setting.clone()} player={winner}/>  <br/>
                        </>
                    }
                };

                let mut h = vec![];
                for p in config.all_player() {
                    let head = distr.players_head(p);
                    let hand = distr.players_hand(p);
                    h.push(html!{
                        <>
                            <PlayerRepView play_setting={play_setting.clone()} player={p}/> {"..."}
                            {"頭："} {for head.iter().map(|c| html!{<CardView config={config.clone()} card={*c}/>})} {"、"}
                            {"手元："} {for hand.iter().map(|c| html!{<CardView config={config.clone()} card={*c}/>})} {"、"}
                        <br/>
                        </>
                    });
                }

                html! {
                    <>
                    <GameConfigView config={config.clone()}/>
                    <PlayerView config={config.clone()} view={view}/>
                    <HistoryView history={history.clone()} play_setting={self.play_setting.clone()} config={config.clone()}/>
                    {winner}
                    {h}
                    </>
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
            Msg::GameEnd(game) => {
                self.scene = Scene::End(game.distr(), game.is_win().unwrap(), game.history());
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
