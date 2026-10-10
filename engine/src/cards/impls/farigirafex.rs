//! Farigiraf ex (TEF, Tera): Armor Tail — prevent all damage done to this
//! Pokémon by attacks from your opponent's Basic Pokémon ex. Dirty Beam —
//! 160; also 30 damage to 1 of your opponent's Benched Pokémon.
//!
//! Armor Tail is one `Prevent` over `Kind(Damage)` whose cause is a Basic Pokémon ex of the opponent's, read at step 6
//! of the damage calculation (APR C-16). It is an Ability, so a lock turns it off; the Tera rule is the card's own
//! `Prevent` (`TERA_RULE`, a card rule) and still protects the Benched Pokémon then. Dirty Beam's 30 is a Damage event
//! on the chosen Benched Pokémon (no Weakness or Resistance, APR B-08; a protected Pokémon can still be chosen).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Farigirafex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(30), target_damage_mul: 0, calc: DamageCalc::Put, when: Cond::True })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::Holder, EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Cause(CausePred::Pokemon(SlotPred::All(&[SlotPred::Basic, SlotPred::Tag(tag::POKEMON_EX_LOWER)])))]))) },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
