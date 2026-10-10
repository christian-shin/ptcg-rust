//! Iron Crown ex (TEF): Cobalt Command — your Future Pokémon's attacks, except any Iron Crown ex, do 20 more damage to
//! your opponent's Active Pokémon (before applying Weakness and Resistance). Twin Shotels — 50 damage to 2 of your
//! opponent's Pokémon; the damage isn't affected by Weakness or Resistance, or by any effects on those Pokémon.
//!
//! Cobalt Command is `Modifier::DamageDealt` (+20 in the attacker-side pass of the Damage calculation, before Weakness):
//! the attacker is a Future Pokémon other than an Iron Crown ex, the target the Defending Pokémon, positive damage;
//! copies stack. Twin Shotels: exactly min(2, the opponent's Pokémon in play) targets (no cancel), each a Damage event
//! of 50 caused by the attack, with the attack flags `IgnoreDefenderEffects` / `NoWeakness` / `NoResistance`: the
//! effects on those Pokémon are ignored, including a `Prevent` over Damage (APR C-16, Shred).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "IronCrownex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true })),
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoWeakness, value: true })),
            Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::NoResistance, value: true })),
            Step::after_damage(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), max: Num::Min(&Num::Lit(2), &Num::SlotCount(SlotSel::Pokemon(Who::Opp), SlotPred::Any)), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), what: EachWhat::Damage(DamageCalc::Deal), amount: Num::Lit(50), ..EachSlotSpec::DEFAULT })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::DamageDealt(DamageDealtSpec { amount: 20, attacker: SlotPred::All(&[SlotPred::Tag(tag::FUTURE), SlotPred::Not(&SlotPred::Named("Iron Crown ex"))]), needs_damage: true, ..DamageDealtSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
