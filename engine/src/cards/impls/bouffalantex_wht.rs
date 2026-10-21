//! Bouffalant ex (WHT 77): Bouffer - this Pokémon takes 30 less damage from attacks (after applying Weakness and
//! Resistance). Gold Breaker - 100+; if your opponent's Active Pokémon is a Pokémon ex, this attack does 100 more damage.
//!
//! Bouffer is a `DamageTaken` passive of the Ability (Mega Diancie ex PFL 41's Diamond Coat); Gold Breaker's 100 more is
//! read before the damage from the Defending Pokémon (Eternatus SSP 141).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "BouffalantEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::before_damage(more_damage_if(100, Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::PokemonEx)))),
            ],
        },
    ],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 30, subject: SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), from_any_attack: true, ..DamageTakenSpec::DEFAULT }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
