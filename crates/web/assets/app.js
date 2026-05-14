import init, { WebGame } from "../pkg/game_web.js";

const elements = {
  startForm: document.getElementById("start-form"),
  refreshState: document.getElementById("refresh-state"),
  queryForm: document.getElementById("query-form"),
  declareForm: document.getElementById("declare-form"),
  seed: document.getElementById("seed"),
  ai1: document.getElementById("ai-1"),
  ai2: document.getElementById("ai-2"),
  queryTo: document.getElementById("query-to"),
  querySort: document.getElementById("query-sort"),
  declareCards: document.getElementById("declare-cards"),
  statusPill: document.getElementById("status-pill"),
  statusMessage: document.getElementById("status-message"),
  summary: document.getElementById("summary"),
  viewJson: document.getElementById("view-json"),
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
  setStatus("ready", "WASM loaded. Start a game.");
}

elements.startForm.addEventListener("submit", (event) => {
  event.preventDefault();
  withGuard(() => {
    const seed = Number.parseInt(elements.seed.value, 10) || 0;
    const ai = [elements.ai1.value, elements.ai2.value];
    game = new WebGame(seed, JSON.stringify(ai));
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
  elements.summary.innerHTML = "";
  appendSummary(`you: ${state.you}`);
  appendSummary(`current turn: ${state.current_turn}`);
  appendSummary(`your turn: ${state.your_turn}`);
  appendSummary(`winner: ${JSON.stringify(state.winner)}`);

  elements.viewJson.textContent = JSON.stringify(state.info.view, null, 2);
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
