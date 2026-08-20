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
