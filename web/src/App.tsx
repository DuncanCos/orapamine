import { useEffect, useState } from "react";
import { useGameStore } from "./store/gameStore";
import { ensureWasmReady, getCatalog } from "./wasmClient";
import type { PieceCatalog } from "./types/protocol";
import { Lobby } from "./components/Lobby";
import { PlacementScreen } from "./components/PlacementScreen";
import { GameScreen } from "./components/GameScreen";
import { ResultScreen } from "./components/ResultScreen";
import { Logo } from "./components/Logo";
import { t } from "./i18n/fr";
import "./styles/theme.css";

export default function App() {
  const [catalog, setCatalog] = useState<PieceCatalog | null>(null);
  const connect = useGameStore((s) => s.connect);
  const tryRestoreSession = useGameStore((s) => s.tryRestoreSession);
  const connected = useGameStore((s) => s.connected);
  const phase = useGameStore((s) => s.phase);
  const code = useGameStore((s) => s.code);
  const errorMessage = useGameStore((s) => s.errorMessage);
  const clearError = useGameStore((s) => s.clearError);
  const leaveGame = useGameStore((s) => s.leaveGame);

  function handleLeaveGame() {
    const mustConfirm = phase === "placement" || phase === "playing";
    if (mustConfirm && !window.confirm(t("app.leave_confirm"))) return;
    leaveGame();
  }

  useEffect(() => {
    ensureWasmReady().then(() => setCatalog(getCatalog()));
    tryRestoreSession();
    connect();
    // Connexion établie une seule fois au montage.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (!catalog) {
    return (
      <div className="app-loading">
        <p>{t("app.title")}…</p>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <header className="app-header">
        <h1 className="app-title">
          <Logo size={32} />
        </h1>
        <div className="app-header-right">
          {!connected && <span className="conn-banner">{t("conn.disconnected")}</span>}
          {code && (
            <button type="button" className="leave-game-button" onClick={handleLeaveGame}>
              {t("app.leave_game")}
            </button>
          )}
        </div>
      </header>

      {errorMessage && (
        <div className="error-banner" onClick={clearError}>
          {errorMessage}
        </div>
      )}

      <main>
        {phase === "connecting" || phase === "lobby" ? (
          <Lobby />
        ) : phase === "placement" ? (
          <PlacementScreen catalog={catalog} />
        ) : phase === "playing" ? (
          <GameScreen catalog={catalog} />
        ) : (
          <ResultScreen catalog={catalog} />
        )}
      </main>
    </div>
  );
}
