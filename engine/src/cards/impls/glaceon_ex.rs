//! Glaceon ex (PRE 26, Tera): Frost Bullet - 110; this attack also does 30 damage to 1 of your opponent's Benched Pokémon. (Don't
//! apply Weakness and Resistance for Benched Pokémon.) Euclase - Knock Out 1 of your opponent's Pokémon that has exactly 6 damage
//! counters on it.
//!
//! The Tera rule is `TERA_RULE`. Frost Bullet: exactly one Benched Pokémon is chosen at step D (none without a Bench), 30 damage with no
//! Weakness or Resistance after the damage (Cynthia's Milotic DRI 53's form with one target). Euclase has no damage: the player
//! chooses among the opponent's Pokémon that have exactly 6 damage counters (60 damage, `SlotPred::DamageCounters`), and that
//! Pokémon is Knocked Out by the attack's effect (one KnockOut event, resolved at the next state check with every other Knock Out;
//! Alolan Exeggutor ex SSP 133's Swinging Sphene); nothing happens when none has exactly 6.
use crate::spec::prelude::*;

const BENCH: SlotSel = SlotSel::Bench(Who::Opp);
const SIX_COUNTERS: SlotSel = SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::DamageCounters(6));

pub static SPEC: CardSpec = CardSpec {
    class: "GlaceonEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec {
                among: BENCH,
                choose: Some(ChooseN { chooser: Who::Me, min: Num::Min(&Num::Lit(1), &Num::SlotCount(BENCH, SlotPred::Any)), max: Num::Min(&Num::Lit(1), &Num::SlotCount(BENCH, SlotPred::Any)) }),
                what: EachWhat::Damage(DamageCalc::Put),
                amount: Num::Lit(30),
                ..EachSlotSpec::DEFAULT
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::PickSlot(PickSlotOp { pick: PickSlotSpec { chooser: Who::Me, among: SIX_COUNTERS }, purpose: PromptPurpose::DamageTarget })),
                Step::after_damage(Op::KnockOut(KnockOutSpec { target: SlotExpr::Picked, when: Cond::True })),
            ],
        },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
