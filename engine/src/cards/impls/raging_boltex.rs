//! Raging Bolt ex (TEF): Burst Roar — discard your hand and draw 6 cards.
//! Bellowing Thunder — you may discard any amount of Basic Energy from your
//! Pokémon; 70 damage for each card discarded.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "RagingBoltex",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::All, ..DiscardSpec::DEFAULT })),
                Step::after_damage(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(6)) })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Among(AmongSpec { all_pokemon: true, which: AmongWhich::Basic, max: None, damage_per: 70, after_damage: false }), ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
