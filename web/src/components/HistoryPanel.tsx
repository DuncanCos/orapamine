import type { HistoryEntry } from "../types/protocol";
import { t } from "../i18n/fr";

interface HistoryPanelProps {
  history: HistoryEntry[];
  yourIndex: number;
  playerNames: string[];
  onHoverEntry?: (entry: HistoryEntry | null) => void;
}

function resultLabel(entry: HistoryEntry): string {
  if (entry.kind === "Beam") {
    if (entry.outcome.kind === "Exit") {
      return `${entry.entry} → ${entry.outcome.point} (${t(`color.${entry.outcome.color}` as never)})`;
    }
    if (entry.outcome.kind === "Absorbed") return `${entry.entry} → ${t("color.absorbed")}`;
    return `${entry.entry} → ${t("color.lost")}`;
  }
  if (entry.kind === "Probe") {
    const r = entry.result;
    const label = r === "empty" ? "vide" : r === "occupied_no_color" ? "gemme sans couleur" : t(`color.${r.color}` as never);
    return `(${entry.x},${entry.y}) → ${label}`;
  }
  if (entry.kind === "Solution") {
    return entry.correct ? "solution correcte !" : "solution incorrecte";
  }
  return t("game.history.timeout");
}

export function HistoryPanel({ history, yourIndex, playerNames, onHoverEntry }: HistoryPanelProps) {
  if (history.length === 0) {
    return (
      <div className="history-panel">
        <h3>{t("game.history.title")}</h3>
        <p className="muted">{t("game.history.empty")}</p>
      </div>
    );
  }
  return (
    <div className="history-panel">
      <h3>{t("game.history.title")}</h3>
      <ul>
        {history
          .slice()
          .reverse()
          .map((entry, i) => (
            <li
              key={history.length - 1 - i}
              className={entry.actor === yourIndex ? "history-mine" : "history-theirs"}
              onMouseEnter={() => onHoverEntry?.(entry)}
              onMouseLeave={() => onHoverEntry?.(null)}
            >
              <span className="history-actor">{playerNames[entry.actor] ?? `J${entry.actor + 1}`}</span>
              <span className="history-kind">
                {entry.kind === "Beam" ? t("game.history.beam") : entry.kind === "Probe" ? t("game.history.probe") : ""}
              </span>
              <span className="history-result">{resultLabel(entry)}</span>
            </li>
          ))}
      </ul>
    </div>
  );
}
