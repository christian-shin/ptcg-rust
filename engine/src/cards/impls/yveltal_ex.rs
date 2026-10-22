//! Yveltal ex (POR 53): Soul Destroyer - Knock Out each of your opponent's Pokémon that has 50 HP or less remaining. Dark Strike -
//! 210; during your next turn, this Pokémon can't use Dark Strike.
//!
//! Soul Destroyer: each Pokémon of the opponent (Active and Bench) whose remaining HP (as the game checks it) is 50 or less is Knocked
//! Out by the attack's effect (one KnockOut event each, resolved at the next state check with every other Knock Out; Hydrapple DRI 18).
//! Dark Strike arms the lasting lock on this Pokémon over the named attack (APR C-15; Scyther TEF).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "YveltalEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::ForEach(ForEachSpec {
                over: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::RemainingHpAtMost(50)),
                body: &[Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::Picked, when: Cond::True }))],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
