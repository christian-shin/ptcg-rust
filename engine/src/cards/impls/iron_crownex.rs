//! Iron Crown ex (TEF): Cobalt Command — your Future Pokémon's attacks,
//! except any Iron Crown ex, do 20 more damage to your opponent's Active
//! Pokémon (before applying Weakness and Resistance). Twin Shotels — 50
//! damage to 2 of your opponent's Pokémon; the damage isn't affected by
//! Weakness or Resistance, or by any effects on those Pokémon.
//!
//! Twinleaf: Cobalt Command reacts to every DealDamageEffect of a player that
//! has this card in play (copies stack): attack phase, a Future source that
//! isn't named Iron Crown ex, the Defending Active as target, positive
//! damage, ability not blocked for that player. Twin Shotels opens a
//! non-cancellable ChoosePokemonPrompt for exactly min(2, the opponent's
//! Pokémon in play) targets (phase 4b: it allowed 1); for each chosen Pokémon
//! it runs a DealDamageEffect of 50 on that Pokémon with `ignoreDefenderEffects`
//! and Weakness/Resistance ignored on the attack (phase 4b R7B: it used to add
//! the 50 straight to the Pokémon, skipping the attacker's effects, e.g.
//! Maximum Belt on the Active ex, and the survive-on-10 effects).
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
