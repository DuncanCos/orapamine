// Génère un pseudo par défaut lisible et sans configuration : deux mots
// (thème mine/gemmes/ondes, en écho à l'univers du jeu) séparés par un
// tiret, ex. "rubis-eclair".

const WORDS_A = [
  "rubis",
  "saphir",
  "topaze",
  "opale",
  "jade",
  "cristal",
  "onyx",
  "grenat",
  "ambre",
  "quartz",
  "diamant",
  "corail",
];

const WORDS_B = [
  "eclair",
  "mineur",
  "prisme",
  "sondeur",
  "reflet",
  "onde",
  "veine",
  "faille",
  "geode",
  "pepite",
  "forage",
  "lueur",
];

function pick(words: string[]): string {
  return words[Math.floor(Math.random() * words.length)];
}

export function generateRandomPseudo(): string {
  return `${pick(WORDS_A)}-${pick(WORDS_B)}`;
}
