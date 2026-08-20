import { useMemo, useState } from "react";
import { useGameStore } from "../store/gameStore";
import { Board, type PointHighlight, type ProbeMarker } from "./Board";
import { PlacementEditor } from "./PlacementEditor";
import { HistoryPanel } from "./HistoryPanel";
import { ColorLegend } from "./ColorLegend";
import { localFireBeam, localProbe } from "../wasmClient";
import type { GemColor, HistoryEntry, PieceCatalog } from "../types/protocol";
import { t } from "../i18n/fr";

interface GameScreenProps {
  catalog: PieceCatalog;
}

type ActionMode = "beam" | "probe";

const REACTIONS = [
  { id: "gg", key: "game.reactions.gg" },
  { id: "oops", key: "game.reactions.oops" },
  { id: "think", key: "game.reactions.think" },
] as const;

export function GameScreen({ catalog }: GameScreenProps) {
  const yourIndex = useGameStore((s) => s.yourIndex);
  const currentTurn = useGameStore((s) => s.currentTurn);
  const players = useGameStore((s) => s.players);
  const history = useGameStore((s) => s.history);
  const options = useGameStore((s) => s.options);
  const suddenDeath = useGameStore((s) => s.suddenDeath);
  const ownPlacement = useGameStore((s) => s.ownPlacement);
  const hypothesis = useGameStore((s) => s.hypothesis);
  const setHypothesis = useGameStore((s) => s.setHypothesis);
  const submitSolution = useGameStore((s) => s.submitSolution);
  const checkHypothesis = useGameStore((s) => s.checkHypothesis);
  const hypothesisCheck = useGameStore((s) => s.hypothesisCheck);
  const fireBeam = useGameStore((s) => s.fireBeam);
  const probe = useGameStore((s) => s.probe);
  const sendReaction = useGameStore((s) => s.sendReaction);

  const [mode, setMode] = useState<ActionMode>("beam");
  const [hoveredEntry, setHoveredEntry] = useState<HistoryEntry | null>(null);

  const isSolo = players.length === 1;
  const opponentIndex = yourIndex === null ? null : yourIndex === 0 ? 1 : 0;
  const isYourTurn = yourIndex !== null && currentTurn === yourIndex;
  const playerNames = players.map((p) => p.pseudo);

  const yourShotsOnOpponent = useMemo(
    () => history.filter((h) => h.actor === yourIndex),
    [history, yourIndex],
  );
  const opponentShotsOnYou = useMemo(
    () => history.filter((h) => h.actor === opponentIndex),
    [history, opponentIndex],
  );

  const usedPointIds = new Set(
    yourShotsOnOpponent.filter((h): h is HistoryEntry & { kind: "Beam" } => h.kind === "Beam").map((h) => h.entry),
  );
  const opponentMarkers: ProbeMarker[] = yourShotsOnOpponent
    .filter((h): h is HistoryEntry & { kind: "Probe" } => h.kind === "Probe")
    .map((h) => ({
      x: h.x,
      y: h.y,
      kind: h.result === "empty" ? "empty" : h.result === "occupied_no_color" ? "occupied_no_color" : "color",
      color: typeof h.result === "object" ? (h.result.color as GemColor) : undefined,
    }));
  const yourMarkers: ProbeMarker[] = opponentShotsOnYou
    .filter((h): h is HistoryEntry & { kind: "Probe" } => h.kind === "Probe")
    .map((h) => ({
      x: h.x,
      y: h.y,
      kind: h.result === "empty" ? "empty" : h.result === "occupied_no_color" ? "occupied_no_color" : "color",
      color: typeof h.result === "object" ? (h.result.color as GemColor) : undefined,
    }));

  const highlightPoints: PointHighlight[] = [];
  const shown = hoveredEntry ?? [...yourShotsOnOpponent].reverse().find((h) => h.kind === "Beam");
  if (shown && shown.kind === "Beam" && shown.actor === yourIndex) {
    highlightPoints.push({ id: shown.entry, role: "entry" });
    if (shown.outcome.kind === "Exit") highlightPoints.push({ id: shown.outcome.point, role: "exit" });
  }

  function handlePointClick(id: string) {
    if (!isYourTurn) return;
    fireBeam(id);
  }
  function handleCellClick(x: number, y: number) {
    if (!isYourTurn) return;
    probe(x, y);
  }

  function checkHypothesisLocally() {
    if (!options?.help_mode) {
      checkHypothesis();
      return;
    }
    const contradictions: string[] = [];
    for (const h of yourShotsOnOpponent) {
      if (h.kind === "Beam") {
        const replayed = localFireBeam(hypothesis, h.entry);
        if (JSON.stringify(replayed) !== JSON.stringify(h.outcome)) {
          contradictions.push(`onde ${h.entry}`);
        }
      } else if (h.kind === "Probe") {
        const replayed = localProbe(hypothesis, h.x, h.y);
        if (JSON.stringify(replayed) !== JSON.stringify(h.result)) {
          contradictions.push(`sondage (${h.x},${h.y})`);
        }
      }
    }
    setLocalCheck({ consistent: contradictions.length === 0, contradicting: contradictions });
  }
  const [localCheckResult, setLocalCheck] = useState<{ consistent: boolean; contradicting: string[] } | null>(null);
  const displayedCheck = localCheckResult ?? hypothesisCheck;

  return (
    <div className="game-screen">
      <div className="game-status">
        <span className={isYourTurn ? "turn-you" : "turn-them"}>
          {isYourTurn ? t("game.your_turn") : t("game.opponent_turn")}
        </span>
        {suddenDeath && <span className="sudden-death">{t("game.sudden_death")}</span>}
        <span className="lives">
          {players.map((p, i) => (
            <span key={i}>
              {p.pseudo}: {"♦".repeat(p.lives)}
            </span>
          ))}
        </span>
      </div>

      <section className="game-board-section">
        <h3>{isSolo ? "Grille adverse" : playerNames[opponentIndex ?? 1] ?? "Adversaire"}</h3>
        {isYourTurn && (
          <div className="action-mode-select">
            <button type="button" className={mode === "beam" ? "active" : ""} onClick={() => setMode("beam")}>
              {t("game.action.beam")}
            </button>
            <button type="button" className={mode === "probe" ? "active" : ""} onClick={() => setMode("probe")}>
              {t("game.action.probe")}
            </button>
          </div>
        )}
        <p className="muted">
          {isYourTurn ? (mode === "beam" ? t("game.action.pick_point") : t("game.action.pick_cell")) : " "}
        </p>
        <Board
          catalog={catalog}
          placements={[]}
          showPoints
          usedPointIds={usedPointIds}
          onPointClick={isYourTurn && mode === "beam" ? handlePointClick : undefined}
          onCellClick={isYourTurn && mode === "probe" ? handleCellClick : undefined}
          markers={opponentMarkers}
          highlightPoints={highlightPoints}
        />
      </section>

      <section className="game-board-section game-board-own">
        <h3>{t("result.boards.yours")}</h3>
        <Board catalog={catalog} placements={ownPlacement} markers={yourMarkers} />
      </section>

      <section className="hypothesis-section">
        <h3>{t("game.hypothesis.title")}</h3>
        <p className="muted">{t("game.action.submit_hint")}</p>
        <PlacementEditor
          catalog={catalog}
          pieces={hypothesis}
          onChange={(p) => {
            setHypothesis(p);
            setLocalCheck(null);
          }}
          diamond={options?.diamond ?? false}
          black={options?.black ?? false}
        />
        <div className="hypothesis-actions">
          {options?.help_mode && (
            <button type="button" onClick={checkHypothesisLocally}>
              {t("game.hypothesis.check")}
            </button>
          )}
          <button type="button" className="primary" disabled={!isYourTurn} onClick={submitSolution}>
            {t("game.action.solution")}
          </button>
        </div>
        {displayedCheck && (
          <p className={displayedCheck.consistent ? "hint-ok" : "hint-bad"}>
            {displayedCheck.consistent
              ? t("game.hypothesis.consistent")
              : `${t("game.hypothesis.inconsistent")} ${displayedCheck.contradicting.join(", ")}`}
          </p>
        )}
      </section>

      <HistoryPanel history={history} yourIndex={yourIndex ?? 0} playerNames={playerNames} onHoverEntry={setHoveredEntry} />
      <ColorLegend />

      {!isSolo && (
        <div className="reactions-bar">
          {REACTIONS.map((r) => (
            <button key={r.id} type="button" onClick={() => sendReaction(r.id)}>
              {t(r.key)}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
