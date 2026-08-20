//! Mélange des couleurs de faisceau (§2.5 du cahier des charges) :
//! fonction pure, testée pour chacune des 16 combinaisons possibles.

use crate::pieces::GemColor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Couleur finale annoncée pour un faisceau, après mélange de toutes les
/// couleurs de gemmes touchées en cours de trajet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultColor {
    Transparent,
    Red,
    Blue,
    Yellow,
    White,
    Purple,
    Orange,
    Green,
    Pink,
    SkyBlue,
    Lemon,
    Black,
    LightPurple,
    LightGreen,
    LightOrange,
    Gray,
}

impl ResultColor {
    pub fn label_fr(self) -> &'static str {
        match self {
            ResultColor::Transparent => "Transparent",
            ResultColor::Red => "Rouge",
            ResultColor::Blue => "Bleu",
            ResultColor::Yellow => "Jaune",
            ResultColor::White => "Blanc",
            ResultColor::Purple => "Violet",
            ResultColor::Orange => "Orange",
            ResultColor::Green => "Vert",
            ResultColor::Pink => "Rose",
            ResultColor::SkyBlue => "Bleu ciel",
            ResultColor::Lemon => "Citron",
            ResultColor::Black => "Noir",
            ResultColor::LightPurple => "Violet clair",
            ResultColor::LightGreen => "Vert clair",
            ResultColor::LightOrange => "Orange clair",
            ResultColor::Gray => "Gris",
        }
    }
}

/// Mélange un ensemble de couleurs de gemmes touchées en une couleur finale
/// de faisceau. Une couleur ne compte qu'une fois, l'ordre n'a pas
/// d'importance (`BTreeSet` le garantit).
pub fn mix_colors(colors: &BTreeSet<GemColor>) -> ResultColor {
    use GemColor::*;
    let r = colors.contains(&Red);
    let b = colors.contains(&Blue);
    let y = colors.contains(&Yellow);
    let w = colors.contains(&White);

    match (r, b, y, w) {
        (false, false, false, false) => ResultColor::Transparent,
        (true, false, false, false) => ResultColor::Red,
        (false, true, false, false) => ResultColor::Blue,
        (false, false, true, false) => ResultColor::Yellow,
        (false, false, false, true) => ResultColor::White,
        (true, true, false, false) => ResultColor::Purple,
        (true, false, true, false) => ResultColor::Orange,
        (false, true, true, false) => ResultColor::Green,
        (true, false, false, true) => ResultColor::Pink,
        (false, true, false, true) => ResultColor::SkyBlue,
        (false, false, true, true) => ResultColor::Lemon,
        (true, true, true, false) => ResultColor::Black,
        (true, true, false, true) => ResultColor::LightPurple,
        (false, true, true, true) => ResultColor::LightGreen,
        (true, false, true, true) => ResultColor::LightOrange,
        (true, true, true, true) => ResultColor::Gray,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use GemColor::*;

    fn set(colors: &[GemColor]) -> BTreeSet<GemColor> {
        colors.iter().copied().collect()
    }

    #[test]
    fn none_is_transparent() {
        assert_eq!(mix_colors(&set(&[])), ResultColor::Transparent);
    }

    #[test]
    fn single_colors() {
        assert_eq!(mix_colors(&set(&[Red])), ResultColor::Red);
        assert_eq!(mix_colors(&set(&[Blue])), ResultColor::Blue);
        assert_eq!(mix_colors(&set(&[Yellow])), ResultColor::Yellow);
        assert_eq!(mix_colors(&set(&[White])), ResultColor::White);
    }

    #[test]
    fn two_colors() {
        assert_eq!(mix_colors(&set(&[Red, Blue])), ResultColor::Purple);
        assert_eq!(mix_colors(&set(&[Red, Yellow])), ResultColor::Orange);
        assert_eq!(mix_colors(&set(&[Blue, Yellow])), ResultColor::Green);
        assert_eq!(mix_colors(&set(&[Red, White])), ResultColor::Pink);
        assert_eq!(mix_colors(&set(&[Blue, White])), ResultColor::SkyBlue);
        assert_eq!(mix_colors(&set(&[Yellow, White])), ResultColor::Lemon);
    }

    #[test]
    fn three_colors() {
        assert_eq!(mix_colors(&set(&[Red, Blue, Yellow])), ResultColor::Black);
        assert_eq!(
            mix_colors(&set(&[Red, Blue, White])),
            ResultColor::LightPurple
        );
        assert_eq!(
            mix_colors(&set(&[Blue, Yellow, White])),
            ResultColor::LightGreen
        );
        assert_eq!(
            mix_colors(&set(&[Red, Yellow, White])),
            ResultColor::LightOrange
        );
    }

    #[test]
    fn four_colors() {
        assert_eq!(
            mix_colors(&set(&[Red, Blue, Yellow, White])),
            ResultColor::Gray
        );
    }

    #[test]
    fn order_and_duplicates_do_not_matter() {
        let mut s1 = BTreeSet::new();
        s1.insert(Red);
        s1.insert(Blue);
        s1.insert(Red); // doublon, ignoré par le Set
        let mut s2 = BTreeSet::new();
        s2.insert(Blue);
        s2.insert(Red);
        assert_eq!(mix_colors(&s1), mix_colors(&s2));
    }
}
