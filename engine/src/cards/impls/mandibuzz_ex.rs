//! Mandibuzz ex (ASC 139): Bone Shot - this attack does 50 damage to 1 of your opponent's Pokémon (no Weakness or Resistance on a Benched Pokémon). Vulture Claw - 160; discard a random card from your opponent's hand.
//!
//! Bone Shot is one `DamageSlot` on a Pokémon the attacker picks among the opponent's, `DamageCalc::Auto` (Cramorant TWM 58); Vulture Claw is Scraggy ASC 134's Knock Off.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MandibuzzEx",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::DamageSlot(DamageSlotSpec {
                    target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp) }),
                    hp: Num::Lit(50),
                    target_damage_mul: 0,
                    calc: DamageCalc::Auto,
                    when: Cond::True,
                })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Hand), cards: CardSel::Random(Num::Lit(1)), ..DiscardSpec::DEFAULT })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
