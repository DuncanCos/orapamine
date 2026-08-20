// État global du client, géré avec zustand. Un seul `GameSocket` module-
// level (pas dans le state) gère la connexion ; ce store traduit les
// messages serveur reçus en état React et expose des actions typées pour
// chaque commande du protocole.

import { create } from "zustand";
import { GameSocket, defaultWsUrl } from "../net/ws";
import type {
  BeamOutcome,
  HistoryEntry,
  Phase,
  PlacedPiece,
  PlayerPublicInfo,
  ProbeOutcome,
  RoomOptions,
  Violation,
} from "../types/protocol";

interface GameOverState {
  winner: number | null;
  boards: PlacedPiece[][];
  history: HistoryEntry[];
}

interface HypothesisCheckState {
  consistent: boolean;
  contradicting: string[];
}

interface ReactionToast {
  playerIndex: number;
  id: string;
  at: number;
}

interface GameState {
  connected: boolean;
  code: string | null;
  yourIndex: number | null;
  players: PlayerPublicInfo[];
  phase: Phase | "connecting";
  currentTurn: number | null;
  firstPlayerIndex: number | null;
  suddenDeath: boolean;
  options: RoomOptions | null;
  history: HistoryEntry[];
  violations: Violation[];
  lastBeamResult: { entry: string; outcome: BeamOutcome } | null;
  lastProbeResult: { x: number; y: number; result: ProbeOutcome } | null;
  lastSolutionCorrect: boolean | null;
  ownPlacement: PlacedPiece[];
  hypothesis: PlacedPiece[];
  hypothesisCheck: HypothesisCheckState | null;
  gameOver: GameOverState | null;
  errorMessage: string | null;
  reaction: ReactionToast | null;
  revealedBoard: PlacedPiece[] | null;

  connect(): void;
  tryRestoreSession(): boolean;
  createGame(pseudo: string, options: RoomOptions): void;
  joinGame(code: string, pseudo: string): void;
  startSolo(pseudo: string, options: RoomOptions): void;
  leaveGame(): void;
  revealSolution(): void;

  setOwnPlacement(pieces: PlacedPiece[]): void;
  requestRandomPlacement(): void;
  submitPlacement(): void;
  readyToPlay(): void;

  fireBeam(entry: string): void;
  probe(x: number, y: number): void;

  setHypothesis(pieces: PlacedPiece[]): void;
  submitSolution(): void;
  checkHypothesis(): void;

  sendReaction(id: string): void;
  requestRematch(): void;

  clearError(): void;
}

let socket: GameSocket | null = null;

function getSocket(): GameSocket {
  if (!socket) {
    socket = new GameSocket(defaultWsUrl());
  }
  return socket;
}

export const useGameStore = create<GameState>((set, get) => ({
  connected: false,
  code: null,
  yourIndex: null,
  players: [],
  phase: "connecting",
  currentTurn: null,
  firstPlayerIndex: null,
  suddenDeath: false,
  options: null,
  history: [],
  violations: [],
  lastBeamResult: null,
  lastProbeResult: null,
  lastSolutionCorrect: null,
  ownPlacement: [],
  hypothesis: [],
  hypothesisCheck: null,
  gameOver: null,
  errorMessage: null,
  reaction: null,
  revealedBoard: null,

  connect() {
    const s = getSocket();
    s.onConnectionChange((connected) => set({ connected }));
    s.onMessage((msg) => {
      switch (msg.type) {
        case "GameCreated":
          set({ code: msg.code, yourIndex: msg.player_index });
          break;
        case "Joined":
          set({ yourIndex: msg.player_index });
          break;
        case "StateUpdate":
          set((state) => ({
            phase: msg.phase,
            players: msg.players,
            yourIndex: msg.your_index,
            currentTurn: msg.current_turn,
            firstPlayerIndex: msg.first_player_index,
            suddenDeath: msg.sudden_death,
            options: msg.options,
            history: msg.history,
            // Restaure le placement déjà soumis après une reconnexion
            // (rechargement de page, coupure réseau) : le serveur le
            // renvoie car il n'est jamais secret pour son propre auteur.
            // On ne l'applique que si le client n'a pas déjà une édition
            // locale en cours, pour ne pas écraser des modifications non
            // encore soumises.
            ownPlacement:
              state.ownPlacement.length === 0 && msg.your_placement
                ? msg.your_placement
                : state.ownPlacement,
            // Une revanche vide l'historique : la solution précédemment
            // révélée (mode solo) ne correspond plus au nouveau plateau.
            revealedBoard: msg.history.length === 0 ? null : state.revealedBoard,
          }));
          break;
        case "PlacementRejected":
          set({ violations: msg.violations });
          break;
        case "RandomPlacement":
          set({ ownPlacement: msg.pieces, violations: [] });
          break;
        case "BeamResult":
          set({ lastBeamResult: { entry: msg.entry, outcome: msg.outcome } });
          break;
        case "ProbeResult":
          set({ lastProbeResult: { x: msg.x, y: msg.y, result: msg.result } });
          break;
        case "SolutionResult":
          set({ lastSolutionCorrect: msg.correct });
          break;
        case "HypothesisCheckResult":
          set({
            hypothesisCheck: {
              consistent: msg.consistent,
              contradicting: msg.contradicting_entries,
            },
          });
          break;
        case "GameOver":
          set({
            gameOver: { winner: msg.winner, boards: msg.boards, history: msg.history },
          });
          break;
        case "ReactionReceived":
          set({ reaction: { playerIndex: msg.player_index, id: msg.id, at: Date.now() } });
          break;
        case "SolutionRevealed":
          set({ revealedBoard: msg.board });
          break;
        case "Error":
          set({ errorMessage: msg.message });
          break;
      }
    });
    s.connect();
  },

  tryRestoreSession() {
    const s = getSocket();
    const token = s.restoreToken();
    if (token) {
      // La reconnexion effective se fait à l'ouverture du socket (voir
      // GameSocket.connect) ; ici on signale juste qu'on a un jeton à
      // essayer, pour que l'UI puisse afficher un état "connexion…".
      set({ phase: "connecting" });
      return true;
    }
    return false;
  },

  createGame(pseudo, options) {
    getSocket().send({ type: "CreateGame", pseudo, options });
  },
  joinGame(code, pseudo) {
    getSocket().send({ type: "JoinGame", code, pseudo });
  },
  startSolo(pseudo, options) {
    getSocket().send({ type: "StartSolo", pseudo, options });
  },

  leaveGame() {
    getSocket().leaveGame();
    set({
      code: null,
      yourIndex: null,
      players: [],
      phase: "lobby",
      currentTurn: null,
      firstPlayerIndex: null,
      suddenDeath: false,
      options: null,
      history: [],
      violations: [],
      lastBeamResult: null,
      lastProbeResult: null,
      lastSolutionCorrect: null,
      ownPlacement: [],
      hypothesis: [],
      hypothesisCheck: null,
      gameOver: null,
      errorMessage: null,
      reaction: null,
      revealedBoard: null,
    });
  },

  revealSolution() {
    getSocket().send({ type: "RevealSolution" });
  },

  setOwnPlacement(pieces) {
    set({ ownPlacement: pieces, violations: [] });
  },
  requestRandomPlacement() {
    getSocket().send({ type: "RequestRandomPlacement" });
  },
  submitPlacement() {
    getSocket().send({ type: "SetPlacement", pieces: get().ownPlacement });
  },
  readyToPlay() {
    getSocket().send({ type: "ReadyToPlay" });
  },

  fireBeam(entry) {
    getSocket().send({ type: "FireBeam", entry });
  },
  probe(x, y) {
    getSocket().send({ type: "Probe", x, y });
  },

  setHypothesis(pieces) {
    set({ hypothesis: pieces, hypothesisCheck: null });
  },
  submitSolution() {
    getSocket().send({ type: "SubmitSolution", pieces: get().hypothesis });
  },
  checkHypothesis() {
    getSocket().send({ type: "CheckHypothesis", pieces: get().hypothesis });
  },

  sendReaction(id) {
    getSocket().send({ type: "Reaction", id });
  },
  requestRematch() {
    getSocket().send({ type: "RequestRematch" });
  },

  clearError() {
    set({ errorMessage: null });
  },
}));
