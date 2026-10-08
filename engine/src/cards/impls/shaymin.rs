//! Shaymin (DRI): Flower Curtain — prevent all damage done to your Benched
//! Pokémon that don't have a Rule Box by attacks from your opponent's
//! Pokémon. Smash Kick — 30.
//!
//! Twinleaf: every Shaymin instance (any zone) reacts to PutDamageEffect;
//! "in play" means any Shaymin on the defending player's board; the lock probe
//! runs on the reacting instance; Rule Box is checked over all cards of the
//! target slot.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Shaymin",
    passives: &[Passive {
        origin: RuleSource::Ability,
        // Your Benched Pokémon that don't have a Rule Box.
        modifier: Modifier::PreventDamage(PreventDamageSpec {
            subject: SlotPred::All(&[SlotPred::IsBench, SlotPred::Not(&SlotPred::RuleBox)]),
            side: Side::Owner,
            ..PreventDamageSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
