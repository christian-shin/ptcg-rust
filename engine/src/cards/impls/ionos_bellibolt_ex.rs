//! Iono's Bellibolt ex (JTG 53): Electric Streamer — as often as you like during your turn, you may attach a Basic [L] Energy card
//! from your hand to 1 of your Iono's Pokémon. Thunderous Bolt — 230; during your next turn, this Pokémon can't attack.
//!
//! Each use of Electric Streamer attaches exactly 1 Basic [L] Energy card from the hand (an Attach event caused by the Ability; not
//! the turn's Energy attachment) to the Active or a Benched Pokémon that has "Iono's" in its name (the Iono's tag); it can be used
//! again while a card and a Pokémon remain. Thunderous Bolt arms a lasting lock on this Pokémon's attacks through your next turn
//! (APR C-15).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "IonosBelliboltEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::Attach(AttachSpec {
            from: ZoneRef(Who::Me, Zone::Hand),
            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Provides(ct::LIGHTNING)]),
            slots: AttachSlots::ActiveBench,
            target: SlotPred::Tag(tag::IONOS),
            bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
            ..AttachSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
