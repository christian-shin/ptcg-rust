//! Walrein (SSP): Frigid Fangs — 60; during your opponent's next turn, Pokémon that have 2 or less Energy attached can't
//! attack. (This includes new Pokémon that come into play.) Megaton Fall — 170; this Pokémon also does 50 damage to itself.
//!
//! Frigid Fangs is an effect on each of the opponent's Pokémon (user decision 2026-10-12, reversing the earlier player
//! lock; `Op::ArmEach`): a "can't attack while it has 2 or less Energy attached" lock put on each of them in play when the
//! attack resolves, one ApplyEffect event each, so a Pokémon that prevents the effects of attacks keeps it off (JP FAQ
//! Walrein / Terapagos / Mist Energy: Mist Energy already attached, it can attack; attached afterwards, it can't, id2341),
//! and on each new one that comes into play during that turn (the parenthetical). It ends for a Pokémon that moves to the
//! Bench, evolves or devolves (APR C-15); Walrein leaving the Active Spot changes nothing (id2055: "resides on all of the
//! opponent's Pokémon, not on Walrein"). The Energy attached is the Energy provided (`SlotPred::EnergyAtMost`, read on the attacking Pokémon).
//!
//! The self damage is a Damage event caused by the attack on this Pokémon (no Weakness or Resistance, APR B-08).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Walrein",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::ArmEach(ArmEachSpec {
                side: Who::Opp,
                subject: SlotPred::Any,
                what: Lasting::Lock(LastingLockSpec::on_defending(&LockDecl::on(EventPred::All(&[EventPred::Kind(EventKind::UseAttack), EventPred::Slot(SlotPred::EnergyAtMost(2))]), "BLOCKED_BY_EFFECT"))),
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(self_damage(50))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
