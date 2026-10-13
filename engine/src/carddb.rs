//! Static printed card data (generated in `gen/cards.rs`) and lookups.

use crate::types::*;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Weakness {
    pub card_type: CardType,
    /// `None` means ×2.
    pub value: Option<i32>,
}

#[derive(Debug)]
pub struct Resistance {
    pub card_type: CardType,
    pub value: i32,
}

#[derive(Debug)]
pub struct AttackDef {
    /// Official English name.
    pub name: &'static str,
    pub cost: &'static [CardType],
    pub damage: i32,
    pub damage_calculation: Option<&'static str>,
    pub text: &'static str,
    pub has_effect_fn: bool,
    pub can_use_on_first_turn: bool,
    pub use_on_bench: bool,
    pub gx_attack: bool,
    pub shred_attack: bool,
    pub barrage: bool,
    pub copycat_attack: bool,
}

#[derive(Debug)]
pub struct PowerDef {
    /// Official English name.
    pub name: &'static str,
    pub power_type: u8,
    pub text: &'static str,
    pub has_effect_fn: bool,
    pub use_when_in_play: bool,
    pub use_from_hand: bool,
    pub use_from_hand_to_bench: bool,
    pub use_from_discard: bool,
    pub exempt_from_ability_lock: bool,
    pub exempt_from_initialize: bool,
    pub ability_lock: bool,
    pub barrage: bool,
    pub knocks_out_self: bool,
    pub is_fossil: bool,
}

#[derive(Debug)]
pub struct CardDef {
    /// Full name "<Name> <SET> <NUM>" (official name, international set code
    /// and printing number): the card's key in `data/cards.json`.
    pub full_name: &'static str,
    /// Official English card name.
    pub name: &'static str,
    /// International set code and printing number.
    pub set: &'static str,
    pub set_number: &'static str,
    /// Class of the printed card.
    pub class: &'static str,
    /// Class in the prototype chain that carries the card's logic ("" if none).
    pub behavior: &'static str,
    pub super_type: u8,
    pub regulation_mark: &'static str,
    pub tags: Tags,
    /// Tags in printed order (`card.tags`).
    pub tag_names: &'static [&'static str],
    pub retreat: &'static [CardType],
    pub attacks: &'static [AttackDef],
    pub powers: &'static [PowerDef],
    pub card_type: &'static [CardType],
    pub stage: u8,
    pub hp: i32,
    pub weakness: &'static [Weakness],
    pub resistance: &'static [Resistance],
    pub evolves_from: &'static str,
    pub evolves_to: &'static [&'static str],
    pub evolves_to_stage: &'static [u8],
    pub evolves_from_base: &'static [&'static str],
    pub max_tools: u8,
    pub trainer_type: u8,
    pub first_turn: bool,
    pub attaches_to_opponents_pokemon: bool,
    pub energy_type: u8,
    pub provides: &'static [CardType],
    pub methods: &'static [&'static str],
    /// Treated as a Pokémon by `getPokemons()` (fossils, dolls).
    pub fossil_doll: bool,
    /// Names that count as the same name for the deck-building rule (empty:
    /// none): a deck holds cards of at most one name of the group (ruling
    /// 1073: Lysandre and Boss's Orders; 746 / 747: Professor Sycamore,
    /// Professor Juniper, Professor's Research).
    pub same_name_group: &'static [&'static str],
    /// Position in the deck / discard listing order (super type, Trainer or
    /// Energy subtype, name in UTF-16 order); equal for equal keys.
    pub sort_rank: u16,
}

impl CardDef {
    pub fn super_type(&self) -> SuperType {
        match self.super_type {
            1 => SuperType::Pokemon,
            2 => SuperType::Trainer,
            3 => SuperType::Energy,
            4 => SuperType::Any,
            _ => SuperType::None,
        }
    }
    pub fn is_pokemon(&self) -> bool {
        self.super_type == 1
    }
    pub fn is_trainer(&self) -> bool {
        self.super_type == 2
    }
    pub fn is_energy(&self) -> bool {
        self.super_type == 3
    }
    pub fn stage(&self) -> Stage {
        Stage::from_u8(self.stage)
    }
    pub fn trainer_type(&self) -> TrainerType {
        match self.trainer_type {
            1 => TrainerType::Supporter,
            2 => TrainerType::Stadium,
            3 => TrainerType::Tool,
            _ => TrainerType::Item,
        }
    }
    pub fn energy_type(&self) -> EnergyType {
        if self.energy_type == 1 {
            EnergyType::Special
        } else {
            EnergyType::Basic
        }
    }
    pub fn has_tag(&self, t: u32) -> bool {
        self.tags & tag_bit(t) != 0
    }
    /// "Pokémon ex": the lower-case ex rule (Mega Evolution Pokémon ex and Tera Pokémon ex included).
    pub fn is_pokemon_ex(&self) -> bool {
        self.has_tag(tag::POKEMON_EX_LOWER)
    }
    /// "Pokémon V": Pokémon V, V-STAR, VMAX and V-UNION (id1667), not Radiant Pokémon.
    pub fn is_pokemon_v(&self) -> bool {
        self.has_tag(tag::POKEMON_V) || self.has_tag(tag::POKEMON_VSTAR) || self.has_tag(tag::POKEMON_VMAX) || self.has_tag(tag::POKEMON_VUNION)
    }
    /// The card has a Rule Box: Pokémon ex and Pokémon V ("Pokémon ex, Pokémon V, etc. have Rule Boxes") and the older
    /// Rule Box Pokémon (-GX, -EX, BREAK, Prism Star, Radiant).
    pub fn has_rule_box(&self) -> bool {
        self.is_pokemon_ex()
            || self.is_pokemon_v()
            || self.has_tag(tag::POKEMON_GX)
            || self.has_tag(tag::POKEMON_EX)
            || self.has_tag(tag::BREAK)
            || self.has_tag(tag::PRISM_STAR)
            || self.has_tag(tag::RADIANT)
    }
    pub fn has_method(&self, m: &str) -> bool {
        self.methods.iter().any(|x| *x == m)
    }
}

pub type DefId = u16;

/// `pokemon_flags` bits: a Pokémon card; treated as one by `getPokemons()` (fossils, dolls).
pub const PF_POKEMON: u8 = 1;
pub const PF_FOSSIL: u8 = 2;

/// Whether the card is a Pokémon / fossil (a dense table: the full card record is big, and the board
/// scans only need these two bits).
#[inline]
pub fn pokemon_flags(id: DefId) -> u8 {
    static T: OnceLock<Vec<u8>> = OnceLock::new();
    T.get_or_init(|| cards().iter().map(|d| (d.is_pokemon() as u8) * PF_POKEMON | (d.fossil_doll as u8) * PF_FOSSIL).collect())[id as usize]
}

pub fn cards() -> &'static [CardDef] {
    crate::gen::cards::CARDS
}

pub fn def(id: DefId) -> &'static CardDef {
    &crate::gen::cards::CARDS[id as usize]
}

/// Key of a card: official name, set and printing number ("Grand Tree SCR 136");
/// the same as `full_name`.
pub fn en_key(id: DefId) -> &'static str {
    def(id).full_name
}

/// Official English name ("Grand Tree").
pub fn en_name(id: DefId) -> &'static str {
    def(id).name
}

fn index() -> &'static HashMap<String, DefId> {
    static INDEX: OnceLock<HashMap<String, DefId>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut m: HashMap<String, DefId> = HashMap::new();
        for (i, c) in crate::gen::cards::CARDS.iter().enumerate() {
            m.insert(c.full_name.to_string(), i as DefId);
        }
        // "Name SET" when only one printing in that set has the name.
        let mut short: HashMap<String, Option<DefId>> = HashMap::new();
        for (i, c) in crate::gen::cards::CARDS.iter().enumerate() {
            let s = c.full_name.rsplit_once(' ').map_or(c.full_name, |(s, _)| s).to_string();
            short.entry(s).and_modify(|v| *v = None).or_insert(Some(i as DefId));
        }
        for (s, v) in short {
            if let Some(i) = v {
                m.entry(s).or_insert(i);
            }
        }
        m
    })
}

/// Look a card up by its key ("Grand Tree SCR 136") or, when unique, "Name SET".
pub fn def_by_full_name(name: &str) -> Option<DefId> {
    index().get(name).copied()
}

/// Whether `name` is the full name of this card.
pub fn card_is(d: &CardDef, name: &str) -> bool {
    d.full_name == name
}

#[cfg(test)]
mod key_tests {
    use super::*;

    #[test]
    fn keys_and_short_names() {
        let g = def_by_full_name("Grand Tree SCR 136").unwrap();
        assert_eq!(def_by_full_name("Grand Tree SCR"), Some(g));
        assert_eq!(en_name(g), "Grand Tree");
        assert_eq!(en_key(g), "Grand Tree SCR 136");
        assert_eq!(def(g).full_name, "Grand Tree SCR 136");
    }

    #[test]
    fn full_names_unique_and_resolve() {
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for (i, c) in cards().iter().enumerate() {
            assert!(seen.insert(c.full_name, i).is_none(), "duplicate full_name {}", c.full_name);
            assert_eq!(def_by_full_name(c.full_name), Some(i as DefId), "{}", c.full_name);
        }
    }
}

#[cfg(test)]
mod phrase_tests {
    use super::*;
    use crate::types::{tag, tag_bit};

    /// A Pokémon card record carrying only `tags` (a card the pool and the data don't have, e.g. a Pokémon V).
    fn with_tags(tags: &[u32]) -> CardDef {
        CardDef {
            full_name: "Test",
            name: "Test",
            set: "TST",
            set_number: "1",
            class: "",
            behavior: "",
            super_type: 1,
            regulation_mark: "",
            tags: tags.iter().fold(0, |a, t| a | tag_bit(*t)),
            tag_names: &[],
            retreat: &[],
            attacks: &[],
            powers: &[],
            card_type: &[],
            stage: 0,
            hp: 60,
            weakness: &[],
            resistance: &[],
            evolves_from: "",
            evolves_to: &[],
            evolves_to_stage: &[],
            evolves_from_base: &[],
            max_tools: 1,
            trainer_type: 0,
            first_turn: false,
            attaches_to_opponents_pokemon: false,
            energy_type: 0,
            provides: &[],
            methods: &[],
            fossil_doll: false,
        }
    }

    /// "Pokémon V" is V, VSTAR, VMAX and V-UNION (id1667), "Pokémon ex" the lower-case ex only; a Radiant Pokémon and the
    /// older Rule Box Pokémon have a Rule Box without being either; a plain Pokémon is none of them.
    #[test]
    fn ex_v_and_rule_box_phrases() {
        for t in [tag::POKEMON_V, tag::POKEMON_VSTAR, tag::POKEMON_VMAX, tag::POKEMON_VUNION] {
            let d = with_tags(&[t]);
            assert!(d.is_pokemon_v() && !d.is_pokemon_ex() && d.has_rule_box(), "tag {t}");
        }
        let ex = with_tags(&[tag::POKEMON_EX_LOWER]);
        assert!(ex.is_pokemon_ex() && !ex.is_pokemon_v() && ex.has_rule_box());
        for t in [tag::RADIANT, tag::POKEMON_GX, tag::POKEMON_EX, tag::BREAK, tag::PRISM_STAR] {
            let d = with_tags(&[t]);
            assert!(!d.is_pokemon_ex() && !d.is_pokemon_v() && d.has_rule_box(), "tag {t}");
        }
        let plain = with_tags(&[]);
        assert!(!plain.is_pokemon_ex() && !plain.is_pokemon_v() && !plain.has_rule_box());
    }

    /// The data: every Pokémon ex has a Rule Box (no regulation-mark exception: Black Kyurem ex SSP 48's data has no mark).
    #[test]
    fn every_pokemon_ex_has_a_rule_box() {
        let mut ex = 0;
        for d in cards() {
            if d.is_pokemon_ex() {
                ex += 1;
                assert!(d.has_rule_box(), "{}", d.full_name);
            }
        }
        assert!(ex > 100);
        let bk = def(def_by_full_name("Black Kyurem ex SSP 48").unwrap());
        assert!(bk.regulation_mark.is_empty() && bk.has_rule_box());
    }
}
