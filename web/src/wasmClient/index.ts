// Fine couche au-dessus des bindings générés par wasm-bindgen
// (`src/wasm/`, généré par `npm run build:wasm`, voir README) : charge le
// module une seule fois et désérialise les enveloppes JSON `{ok,value|error}`
// renvoyées par `orapa-wasm`.

import init, {
  catalog_json,
  fire_beam_json,
  probe_json,
  validate_placement_json,
} from "../wasm/orapa_wasm.js";
import type { BeamOutcome, GemColor, PieceCatalog, PlacedPiece, ProbeOutcome, Violation } from "../types/protocol";

let ready: Promise<void> | null = null;

export function ensureWasmReady(): Promise<void> {
  if (!ready) {
    ready = init().then(() => undefined);
  }
  return ready;
}

interface Envelope<T> {
  ok: boolean;
  value?: T;
  error?: string;
}

function unwrap<T>(raw: string): T {
  const env: Envelope<T> = JSON.parse(raw);
  if (!env.ok) {
    throw new Error(env.error ?? "erreur WASM inconnue");
  }
  return env.value as T;
}

export function getCatalog(): PieceCatalog {
  return JSON.parse(catalog_json());
}

export function validatePlacement(pieces: PlacedPiece[]): Violation[] {
  return unwrap<Violation[]>(validate_placement_json(JSON.stringify(pieces)));
}

export function localFireBeam(pieces: PlacedPiece[], entryId: string): BeamOutcome {
  return unwrap<BeamOutcome>(fire_beam_json(JSON.stringify(pieces), entryId));
}

export function localProbe(pieces: PlacedPiece[], x: number, y: number): ProbeOutcome {
  const raw = unwrap<{ kind: string; color?: string }>(probe_json(JSON.stringify(pieces), x, y));
  if (raw.kind === "Empty") return "empty";
  if (raw.kind === "OccupiedNoColor") return "occupied_no_color";
  return { color: raw.color as GemColor };
}
