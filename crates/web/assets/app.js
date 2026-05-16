import init, { WebGame } from "../pkg/game_web.js";

const elements = {
  startForm: document.getElementById("start-form"),
  refreshState: document.getElementById("refresh-state"),
  queryForm: document.getElementById("query-form"),
  declareForm: document.getElementById("declare-form"),
  seed: document.getElementById("seed"),
  randomSeed: document.getElementById("random-seed"),
  userPlayer: document.getElementById("user-player"),
  ai1Label: document.getElementById("ai-1-label"),
  ai2Label: document.getElementById("ai-2-label"),
  ai1: document.getElementById("ai-1"),
  ai2: document.getElementById("ai-2"),
  queryTo: document.getElementById("query-to"),
  querySort: document.getElementById("query-sort"),
  declareCards: document.getElementById("declare-cards"),
  statusPill: document.getElementById("status-pill"),
  statusMessage: document.getElementById("status-message"),
  summary: document.getElementById("summary"),
  configMeta: document.getElementById("config-meta"),
  allCards: document.getElementById("all-cards"),
  viewTableBody: document.getElementById("view-table-body"),
  historyJson: document.getElementById("history-json"),
  possibleMoves: document.getElementById("possible-moves"),
};

let wasmReady = false;
let game = null;
let currentState = null;

boot().catch((error) => {
  setError(`init failed: ${error instanceof Error ? error.message : String(error)}`);
});

async function boot() {
  await init();
  wasmReady = true;
  applyRandomSeed();
  syncAiLabels();
  setStatus("ready", "WASM loaded. Start a game.");
}

elements.randomSeed.addEventListener("click", () => {
  applyRandomSeed();
});

elements.userPlayer.addEventListener("change", () => {
  syncAiLabels();
});

elements.startForm.addEventListener("submit", (event) => {
  event.preventDefault();
  withGuard(() => {
    const seed = Number.parseInt(elements.seed.value, 10) || 0;
    const userPlayer = Number.parseInt(elements.userPlayer.value, 10) || 0;
    const ai = [elements.ai1.value, elements.ai2.value];
    game = new WebGame(seed, userPlayer, JSON.stringify(ai));
    renderState(JSON.parse(game.state_json()));
    setStatus("ready", "Game started.");
  });
});

elements.refreshState.addEventListener("click", () => {
  withGuard(() => {
    assertGame();
    renderState(JSON.parse(game.state_json()));
    setStatus("ready", "State refreshed.");
  });
});

elements.queryForm.addEventListener("submit", (event) => {
  event.preventDefault();
  withGuard(() => {
    assertGame();
    const move = {
      Query: {
        query_to: Number.parseInt(elements.queryTo.value, 10),
        query_sort: elements.querySort.value.trim(),
      },
    };
    renderState(JSON.parse(game.play_move_json(JSON.stringify(move))));
    setStatus("ready", "Query submitted.");
  });
});

elements.declareForm.addEventListener("submit", (event) => {
  event.preventDefault();
  withGuard(() => {
    assertGame();
    const declare = elements.declareCards.value
      .trim()
      .split(/\s+/)
      .filter(Boolean)
      .map((value) => Number.parseInt(value, 10));
    const move = { Declare: { declare } };
    renderState(JSON.parse(game.play_move_json(JSON.stringify(move))));
    setStatus("ready", "Declare submitted.");
  });
});

function renderState(state) {
  currentState = state;
  renderConfig(state.info.config);
  elements.summary.innerHTML = "";
  appendSummary(`you: ${state.you}`);
  appendSummary(`current turn: ${state.current_turn}`);
  appendSummary(`your turn: ${state.your_turn}`);
  appendSummary(`winner: ${JSON.stringify(state.winner)}`);

  renderViewTable(state);
  elements.historyJson.textContent = JSON.stringify(state.info.query_answer, null, 2);

  elements.possibleMoves.innerHTML = "";
  for (const move of state.possible_moves) {
    const chip = document.createElement("button");
    chip.type = "button";
    chip.className = "chip";
    chip.textContent = formatMove(move);
    chip.addEventListener("click", () => fillMoveForm(move));
    elements.possibleMoves.appendChild(chip);
  }
}

function fillMoveForm(move) {
  if (move.Query) {
    elements.queryTo.value = String(move.Query.query_to);
    elements.querySort.value = move.Query.query_sort;
    return;
  }
  if (move.Declare) {
    elements.declareCards.value = move.Declare.declare.join(" ");
  }
}

function formatMove(move) {
  if (move.Query) {
    return `query ${move.Query.query_to} ${move.Query.query_sort}`;
  }
  if (move.Declare) {
    return `declare ${move.Declare.declare.join(" ")}`;
  }
  return JSON.stringify(move);
}

function appendSummary(text) {
  const item = document.createElement("span");
  item.textContent = text;
  elements.summary.appendChild(item);
}

function appendSummaryValue(container, text) {
  const item = document.createElement("span");
  item.textContent = text;
  container.appendChild(item);
}

function renderViewTable(state) {
  const { you, info } = state;
  const rows = info.view.other.map((head, player) => {
    if (player === you) {
      return {
        player,
        hand: formatCards(info.view.hand),
        head: "",
      };
    }
    return {
      player,
      hand: "",
      head: formatCards(head),
    };
  });

  elements.viewTableBody.innerHTML = "";
  for (const row of rows) {
    const tr = document.createElement("tr");
    appendCell(tr, `Player ${row.player}`, "player-col");
    appendCell(tr, row.hand);
    appendCell(tr, row.head);
    elements.viewTableBody.appendChild(tr);
  }
}

function renderConfig(config) {
  elements.configMeta.innerHTML = "";
  appendSummaryValue(elements.configMeta, `players: ${config.player_num}`);
  appendSummaryValue(elements.configMeta, `hand: ${config.hand_num}`);
  appendSummaryValue(elements.configMeta, `head: ${config.head_num}`);
  appendSummaryValue(elements.configMeta, `sorts: ${config.sorts.join(" ")}`);

  elements.allCards.innerHTML = "";
  for (const [index, sorts] of config.cards_sort.entries()) {
    const item = document.createElement("div");
    item.className = "card-token";
    item.textContent = `${index}: ${sorts.join(" ")}`;
    elements.allCards.appendChild(item);
  }
}

function appendCell(tr, text, className = "") {
  const td = document.createElement("td");
  td.textContent = text;
  if (className) {
    td.className = className;
  }
  tr.appendChild(td);
}

function formatCards(cards) {
  if (!cards || cards.length === 0) {
    return "";
  }
  return cards.join(" ");
}

function applyRandomSeed() {
  elements.seed.value = String(Math.floor(Math.random() * 1_000_000_000));
}

function syncAiLabels() {
  const userPlayer = Number.parseInt(elements.userPlayer.value, 10) || 0;
  const aiPlayers = [0, 1, 2].filter((player) => player !== userPlayer);
  elements.ai1Label.textContent = `Player ${aiPlayers[0]}`;
  elements.ai2Label.textContent = `Player ${aiPlayers[1]}`;
}

function assertGame() {
  if (!wasmReady) {
    throw new Error("WASM is not ready yet");
  }
  if (!game) {
    throw new Error("game is not started");
  }
}

function withGuard(fn) {
  try {
    fn();
  } catch (error) {
    setError(error instanceof Error ? error.message : String(error));
  }
}

function setStatus(kind, message) {
  elements.statusPill.className = `pill ${kind}`;
  elements.statusPill.textContent = kind;
  elements.statusMessage.textContent = message;
}

function setError(message) {
  setStatus("error", message);
  if (currentState) {
    renderState(currentState);
  }
}
