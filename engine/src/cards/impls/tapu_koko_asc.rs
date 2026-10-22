//! Tapu Koko (ASC 67): Fast Flight (if you go first, you can use it during your first turn) - discard your hand and draw 5 cards. Thunder Blast - 130; discard 2 Energy from this Pokémon.
//!
//! Fast Flight's first-turn permission is `AttackFlags { first_turn: Attack(0) }` (Delibird MEG 105's Quick Gift), read by
//! the UseAttack's first-turn check; the Discard of the hand and the Draw are effects of the attack (Raging Bolt ex TEF's
//! Burst Roar). Thunder Blast discards 2 Energy after the damage (Metagross TEF 115's Luster Blast).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TapuKokoASC",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, ..DiscardSpec::DEFAULT })),
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(5)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 2, ty: crate::types::ct::COLORLESS, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            ],
        },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: Some(FirstTurnAttack::Attack(0)), shred: None }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
