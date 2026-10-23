//! Wugtrio ex (TEF 60): Tera - as long as this Pokémon is on your Bench, prevent all damage done to this Pokémon by attacks
//! (both yours and your opponent's). Tricolor Pump - discard up to 3 Energy cards from your hand. This attack does 60
//! damage to 1 of your opponent's Pokémon for each Energy card you discarded in this way. (Don't apply Weakness and
//! Resistance for Benched Pokémon.) Numbing Hold - 120; during your opponent's next turn, the Defending Pokémon can't
//! retreat.
//!
//! The Tera rule is `TERA_RULE`. Tricolor Pump chooses 0 to 3 Energy cards of the hand and discards them (a Discard event
//! caused by the attack, "up to" in an attack may be 0, id2301), then chooses 1 of the opponent's Pokémon and does 60 times
//! the cards discarded to it (`DamageCalc::Auto`: Weakness and Resistance only to the Active Pokémon, APR B-08; Cramorant
//! TWM 58). Numbing Hold is the lasting retreat lock on the Defending Pokémon (Gliscor PFL 50).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WugtrioEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::Energy, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, into: 0, ..PickSpec::DEFAULT })),
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                    target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
                    hp: Num::Mul(&Num::RegCount(0), &Num::Lit(60)),
                    target_damage_mul: 0,
                    calc: DamageCalc::Auto,
                    when: Cond::True,
                })),
            ],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
