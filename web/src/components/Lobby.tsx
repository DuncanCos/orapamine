import { useState } from "react";
import { useGameStore } from "../store/gameStore";
import type { RoomOptions } from "../types/protocol";
import { t } from "../i18n/fr";

const DEFAULT_OPTIONS: RoomOptions = {
  diamond: false,
  black: false,
  lives: 1,
  turn_timer_secs: null,
  help_mode: true,
};

export function Lobby() {
  const [pseudo, setPseudo] = useState("");
  const [code, setCode] = useState("");
  const [options, setOptions] = useState<RoomOptions>(DEFAULT_OPTIONS);
  const [error, setError] = useState<string | null>(null);

  const createGame = useGameStore((s) => s.createGame);
  const joinGame = useGameStore((s) => s.joinGame);
  const startSolo = useGameStore((s) => s.startSolo);
  const gameCode = useGameStore((s) => s.code);
  const players = useGameStore((s) => s.players);
  const phase = useGameStore((s) => s.phase);

  function requirePseudo(): boolean {
    if (!pseudo.trim()) {
      setError(t("lobby.error.pseudo_required"));
      return false;
    }
    setError(null);
    return true;
  }

  function handleCreate() {
    if (!requirePseudo()) return;
    createGame(pseudo.trim(), options);
  }

  function handleJoin() {
    if (!requirePseudo()) return;
    if (!code.trim()) {
      setError(t("lobby.error.code_required"));
      return;
    }
    joinGame(code.trim().toUpperCase(), pseudo.trim());
  }

  function handleSolo() {
    if (!requirePseudo()) return;
    startSolo(pseudo.trim(), options);
  }

  if (gameCode && phase === "lobby") {
    return (
      <div className="lobby-waiting">
        <h2>{t("lobby.share_code")}</h2>
        <div className="lobby-code">{gameCode}</div>
        <p className="muted">{t("lobby.waiting_opponent")}</p>
        <p>{players[0]?.pseudo}</p>
      </div>
    );
  }

  return (
    <div className="lobby">
      <h1>{t("lobby.title")}</h1>

      <label className="field">
        {t("lobby.pseudo")}
        <input
          value={pseudo}
          onChange={(e) => setPseudo(e.target.value)}
          placeholder={t("lobby.pseudo.placeholder")}
          maxLength={20}
        />
      </label>

      <fieldset className="lobby-options">
        <legend>{t("lobby.options")}</legend>
        <label>
          <input
            type="checkbox"
            checked={options.diamond}
            onChange={(e) => setOptions((o) => ({ ...o, diamond: e.target.checked }))}
          />
          {t("lobby.options.diamond")}
        </label>
        <label>
          <input
            type="checkbox"
            checked={options.black}
            onChange={(e) => setOptions((o) => ({ ...o, black: e.target.checked }))}
          />
          {t("lobby.options.black")}
        </label>
        <label>
          {t("lobby.options.lives")}
          <select
            value={options.lives}
            onChange={(e) => setOptions((o) => ({ ...o, lives: Number(e.target.value) }))}
          >
            <option value={1}>1</option>
            <option value={2}>2</option>
          </select>
        </label>
        <label>
          {t("lobby.options.timer")}
          <select
            value={options.turn_timer_secs ?? ""}
            onChange={(e) =>
              setOptions((o) => ({
                ...o,
                turn_timer_secs: e.target.value ? Number(e.target.value) : null,
              }))
            }
          >
            <option value="">{t("lobby.options.timer.none")}</option>
            <option value={30}>30s</option>
            <option value={60}>60s</option>
            <option value={120}>120s</option>
          </select>
        </label>
        <label>
          <input
            type="checkbox"
            checked={options.help_mode}
            onChange={(e) => setOptions((o) => ({ ...o, help_mode: e.target.checked }))}
          />
          {t("lobby.options.help_mode")}
        </label>
      </fieldset>

      {error && <p className="error-text">{error}</p>}

      <div className="lobby-actions">
        <button type="button" className="primary" onClick={handleCreate}>
          {t("lobby.create")}
        </button>
        <div className="lobby-join">
          <input
            value={code}
            onChange={(e) => setCode(e.target.value)}
            placeholder={t("lobby.code.placeholder")}
            maxLength={5}
          />
          <button type="button" onClick={handleJoin}>
            {t("lobby.join")}
          </button>
        </div>
        <button type="button" className="secondary" onClick={handleSolo}>
          {t("lobby.solo")}
        </button>
      </div>
    </div>
  );
}
