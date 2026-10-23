//! Cinderace ex (SCR 28, Tera): Flare Strike - 280; during your next turn, this Pokémon can't use Flare Strike. Garnet Volley - this
//! attack does 180 damage to 1 of your opponent's Pokémon. (Don't apply Weakness and Resistance for Benched Pokémon.)
//!
//! Tera: `TERA_RULE` (a `Prevent` over damage while on the Bench). Flare Strike arms a lasting `CANT_ATTACK` lock on this Pokémon
//! naming the attack until the end of your next turn (APR C-15; Koraidon ex ASC's Impact Blow). Garnet Volley chooses one of the
//! opponent's Pokémon at step D and does 180 to it after the (zero) attack damage, with Weakness and Resistance only for the
//! Active Pokémon (APR B-08; `DamageCalc::Auto`, Honchkrow PFL 58).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "CinderaceEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }), hp: Num::Lit(180), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True }))],
        },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
