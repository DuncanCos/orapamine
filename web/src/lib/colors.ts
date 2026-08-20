import type { GemColor, ResultColor } from "../types/protocol";

// Palette pixel-art volontairement contrastée et à teintes plates (pas de
// dégradés) — voir le thème global dans `styles/theme.css`.
export const GEM_COLORS: Record<GemColor, string> = {
  red: "#d3392f",
  yellow: "#e8b93a",
  blue: "#2f6fd3",
  white: "#f2ede1",
};

export const RESULT_COLORS: Record<ResultColor, string> = {
  transparent: "#8fa3ad",
  red: "#d3392f",
  blue: "#2f6fd3",
  yellow: "#e8b93a",
  white: "#f2ede1",
  purple: "#8f4fd3",
  orange: "#e07a2a",
  green: "#4fae5b",
  pink: "#e894b0",
  sky_blue: "#7fc4e8",
  lemon: "#eede7a",
  black: "#2a2622",
  light_purple: "#c3a3e6",
  light_green: "#a8dcae",
  light_orange: "#f0bd8a",
  gray: "#a3a099",
};

export const SPECIAL_COLORS = {
  absorb: "#171513",
  transparent: "#cfeaf0",
};

// Décomposition de chaque couleur de résultat en gemmes traversées — inverse
// de `mix_colors` (crates/orapa-core/src/colors.rs) — pour pouvoir expliquer
// un résultat ("Violet" -> Rouge + Bleu) sans le redériver côté client.
export const RESULT_MIX: Record<ResultColor, GemColor[]> = {
  transparent: [],
  red: ["red"],
  blue: ["blue"],
  yellow: ["yellow"],
  white: ["white"],
  purple: ["red", "blue"],
  orange: ["red", "yellow"],
  green: ["blue", "yellow"],
  pink: ["red", "white"],
  sky_blue: ["blue", "white"],
  lemon: ["yellow", "white"],
  black: ["red", "blue", "yellow"],
  light_purple: ["red", "blue", "white"],
  light_green: ["blue", "yellow", "white"],
  light_orange: ["red", "yellow", "white"],
  gray: ["red", "blue", "yellow", "white"],
};
