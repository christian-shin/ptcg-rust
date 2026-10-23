//! Tornadus (WHT 78): Wrapped in Wind - attach a Basic Energy card from your hand to this Pokémon. Hurricane - 100; move a Basic
//! Energy from this Pokémon to 1 of your Benched Pokémon.
//!
//! Wrapped in Wind: one Basic Energy card is chosen from the hand when there is one (none to attach: nothing happens) and attached
//! to this Pokémon (an Attach event per card, caused by the attack; Teal Mask Ogerpon ex's Teal Dance). Hurricane moves min(1, Basic
//! Energy on this Pokémon) Energy to one Benched Pokémon after the damage (Yanmega ex's Jet Cyclone).
use crate::spec::prelude::*;

const HAND: ZoneRef = ZoneRef(Who::Me, Zone::Hand);
const HAS_BASIC_ENERGY: Cond = Cond::Nonempty(HAND, Pred::BasicEnergy);

pub static SPEC: CardSpec = CardSpec {
    class: "Tornadus",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                    from: HAND,
                    predicate: Pred::BasicEnergy,
                    bounds: Bounds { min: Num::If(&HAS_BASIC_ENERGY, &Num::Lit(1), &Num::Lit(0)), max: Num::Lit(1) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::Attach(AttachSpec { from: HAND, cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec {
                selection: EnergySelection::ToBench {
                    min: Num::Min(&Num::Lit(1), &Num::CardCount(ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE)), Pred::BasicEnergy)),
                    max: Num::Min(&Num::Lit(1), &Num::CardCount(ZoneRef(Who::Me, Zone::Attached(MY_ACTIVE)), Pred::BasicEnergy)),
                    same_target: true,
                    via_effect: false,
                    kind: EnergyKind::Basic,
                },
                to: EnergyDest::Stay,
                ..DiscardEnergySpec::DEFAULT
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
