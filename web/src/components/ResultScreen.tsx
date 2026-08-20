import { useGameStore } from "../store/gameStore";
import { Board } from "./Board";
import { HistoryPanel } from "./HistoryPanel";
import type { PieceCatalog } from "../types/protocol";
import { t } from "../i18n/fr";

interface ResultScreenProps {
  catalog: PieceCatalog;
}

export function ResultScreen({ catalog }: ResultScreenProps) {
  const gameOver = useGameStore((s) => s.gameOver);
  const yourIndex = useGameStore((s) => s.yourIndex);
  const players = useGameStore((s) => s.players);
  const requestRematch = useGameStore((s) => s.requestRematch);

  if (!gameOver || yourIndex === null) return null;

  const outcome =
    gameOver.winner === null ? "draw" : gameOver.winner === yourIndex ? "win" : "lose";

  return (
    <div className="result-screen">
      <h2>{t("result.title")}</h2>
      <p className={`result-outcome result-${outcome}`}>
        {outcome === "win" ? t("result.win") : outcome === "lose" ? t("result.lose") : t("result.draw")}
      </p>

      <div className="result-boards">
        {gameOver.boards.map((board, i) => (
          <div key={i}>
            <h3>{i === yourIndex ? t("result.boards.yours") : players[i]?.pseudo ?? t("result.boards.opponent")}</h3>
            <Board catalog={catalog} placements={board} />
          </div>
        ))}
      </div>

      <HistoryPanel history={gameOver.history} yourIndex={yourIndex} playerNames={players.map((p) => p.pseudo)} />

      {players.length === 2 && (
        <button type="button" className="primary" onClick={requestRematch}>
          {t("result.rematch")}
        </button>
      )}
    </div>
  );
}
