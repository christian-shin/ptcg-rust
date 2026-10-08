//! Heavy Baton (TEF / PAR, tool): if the Pokémon this card is attached to has
//! a Retreat Cost of exactly 4, is in the Active Spot, and is Knocked Out
//! by damage from an attack from your opponent's Pokémon, move up to 3 Basic
//! Energy cards from that Pokémon to your Benched Pokémon in any way you
//! like.
//!
//! Twinleaf: a KnockOutEffect of a slot holding this tool during the
//! opponent's ATTACK phase, where the slot is the owner's Active Spot and the
//! owner carries DAMAGE_DEALT_MARKER (Knocked Out by damage from an attack;
//! fixed in phase 4b, it used to trigger on any KO), and the current Retreat
//! Cost (CheckRetreatCostEffect) is exactly 4 (phase 4b: it used to be a
//! printed Retreat Cost of 4 or more). The Energy list is a copy of the Basic
//! Energy on the slot, but the transfers move the cards from the owner's
//! discard pile (the core has already discarded the Pokémon); the slot marker
//! is removed when the prompt resolves (no cancel, 1 to 3 since phase 4b, any
//! Benched Pokémon in any combination: no sameTarget since phase 4b; nothing
//! happens without a Benched Pokémon).
//!
//! R7C (ruling 1547): the criteria are checked when the damage is dealt. A PutDamageEffect
//! from an opponent's attack on a slot holding this tool refreshes `HEAVY_BATON_ACTIVE_MARKER`
//! (a Trainer-sourced marker, so it survives the switch): set when the slot is the owner's
//! Active Spot, not prevented, damage > 0 and the Retreat Cost is exactly 4. The Knock Out
//! consumes it instead of requiring the Active Spot, so an attack that switches the Pokémon
//! out before the Knock Out is checked (Bayleef's Push Down) still triggers it.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HeavyBaton",
    passives: &[
        Passive { origin: RuleSource::Tool, modifier: Modifier::HeavyBaton(HeavyBatonSpec { retreat_cost: 4, max: 3 }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
