//! Feraligatr (TEF): Torrential Heart — once during your turn, you may put 5
//! damage counters on this Pokémon; attacks used by this Pokémon do 120 more
//! damage this turn. Giant Wave — 160; this Pokémon can't use Giant Wave
//! during your next turn.
//!
//! Torrential Heart puts the 5 counters with one PlaceCounters event (the Ability is its cause; the counters aren't damage
//! and can't be prevented by an attack's protection) and makes the once-per-turn usage record, and puts an effect on this Pokémon for the
//! turn (APR C-15; it ends when the Pokémon evolves or leaves play): the declared `DamageDealt`, +120 to the main damage of any of
//! its attacks at the attack stage (before Weakness and Resistance). Giant
//! Wave's "can't use" is an ApplyEffect event on this Pokémon (`Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This))`).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Feraligatr",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) })),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[],
        steps: &[
            Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::This), counters: Num::Lit(5) })),
            Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Pokemon(SlotExpr::This), until: DeclareUntil::EndOfTurn })),
        ],
    }],
    effects: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::DamageDealt(DamageDealtSpec { stage: DamageStage::Attack, amount: 120, ..DamageDealtSpec::DEFAULT }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
