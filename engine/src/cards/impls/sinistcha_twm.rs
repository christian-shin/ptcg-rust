//! Sinistcha (TWM 22): Cursed Drop - put 4 damage counters on your opponent's Pokémon in any way you like. Spill the Tea -
//! 70×; discard up to 3 [G] Energy cards from your Pokémon. This attack does 70 damage for each card you discarded in this
//! way.
//!
//! Cursed Drop spreads 4 damage counters over the opponent's Pokémon, any of them (one allocation prompt; Dragapult ex
//! TWM's Phantom Dive with the Active Pokémon too; id388). Spill the Tea chooses 0 to 3 [G] Energy cards among all your
//! Pokémon before the damage ("up to" in an attack may be 0, id2301), the damage is 70 per card, and the cards leave play
//! after the damage (Mega Charizard X ex PFL 13's Inferno X).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "SinistchaTWM",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::SpreadDamage(SpreadDamageSpec { chooser: Who::Me, side: Who::Opp, slots: SpreadSlots::Pokemon, total_hp: 40, unit_hp: 10, cap_bonus_hp: None, apply: SpreadApply::Counters }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec {
                target: SlotTarget::Slot(MY_ACTIVE),
                selection: EnergySelection::Among(AmongSpec { all_pokemon: true, which: AmongWhich::Provides(ct::GRASS), max: Some(3), damage_per: 70, after_damage: true }),
                ..DiscardEnergySpec::DEFAULT
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
