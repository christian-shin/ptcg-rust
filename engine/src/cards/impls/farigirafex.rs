//! Farigiraf ex (TEF, Tera): Armor Tail — prevent all damage done to this
//! Pokémon by attacks from your opponent's Basic Pokémon ex. Dirty Beam —
//! 160; also 30 damage to 1 of your opponent's Benched Pokémon.
//!
//! Twinleaf quirk kept: Armor Tail reacts to every PutDamageEffect whose
//! target list contains this card (any source, any phase). Phase 4b: it probes
//! the lock with a real PowerEffect for the *owner* of this Pokémon (it used
//! to be the attacking player), and a locked Ability no longer returns early,
//! so the Tera bench protection (not an Ability) still applies.
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
