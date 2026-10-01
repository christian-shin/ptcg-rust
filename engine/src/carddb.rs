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
    pub full_name: &'static str,
    pub name: &'static str,
    pub set: &'static str,
    pub set_number: &'static str,
    /// Twinleaf class of the printed card.
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
    /// `Card.hasRuleBox()`.
    pub fn has_rule_box(&self) -> bool {
        (self.has_tag(tag::POKEMON_EX_LOWER) && !self.regulation_mark.is_empty())
            || self.has_tag(tag::POKEMON_V)
            || self.has_tag(tag::POKEMON_VMAX)
            || self.has_tag(tag::POKEMON_VSTAR)
            || self.has_tag(tag::POKEMON_VUNION)
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

pub fn cards() -> &'static [CardDef] {
    crate::gen::cards::CARDS
}

pub fn def(id: DefId) -> &'static CardDef {
    &crate::gen::cards::CARDS[id as usize]
}

/// English key of a card: official name, set and printing number
/// ("Grand Tree SCR 136"). Twinleaf's `full_name` stays the oracle identity.
pub fn en_key(id: DefId) -> &'static str {
    crate::gen::names::EN_NAMES[id as usize].0
}

/// Official English name ("Grand Tree"; Twinleaf may say "Great Tree").
pub fn en_name(id: DefId) -> &'static str {
    crate::gen::names::EN_NAMES[id as usize].1
}

fn index() -> &'static HashMap<String, DefId> {
    static INDEX: OnceLock<HashMap<String, DefId>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut m: HashMap<String, DefId> = HashMap::new();
        for (i, c) in crate::gen::cards::CARDS.iter().enumerate() {
            m.insert(c.full_name.to_string(), i as DefId);
        }
        // English aliases: the key, and "Name SET" when only one printing in
        // that set has the name. Twinleaf full names win any collision.
        let mut short: HashMap<String, Option<DefId>> = HashMap::new();
        for (i, (key, _)) in crate::gen::names::EN_NAMES.iter().enumerate() {
            m.entry(key.to_string()).or_insert(i as DefId);
            let s = key.rsplit_once(' ').map_or(*key, |(s, _)| s).to_string();
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

/// Look a card up by Twinleaf full name or English key / "Name SET".
pub fn def_by_full_name(name: &str) -> Option<DefId> {
    index().get(name).copied()
}

#[cfg(test)]
mod en_tests {
    use super::*;

    #[test]
    fn english_aliases() {
        let g = def_by_full_name("Great Tree SCR").unwrap();
        assert_eq!(def_by_full_name("Grand Tree SCR 136"), Some(g));
        assert_eq!(def_by_full_name("Grand Tree SCR"), Some(g));
        assert_eq!(en_name(g), "Grand Tree");
        assert_eq!(en_key(g), "Grand Tree SCR 136");
    }
}
