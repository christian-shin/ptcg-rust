//! Philippe (CRI 79 / M4): attach up to 2 Basic [M] Energy cards from your
//! discard pile to 1 of your [M] Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Philippe",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        // Used as the effect of an attack (Look-Alike Show) it does nothing instead of being refused.
        needs: &[Cond::Any(&[Cond::ViaAttack, Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Provides(crate::types::ct::METAL)])), Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(crate::types::ct::METAL))])])],
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Provides(crate::types::ct::METAL)])), Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(crate::types::ct::METAL))]),
            yes: &[
                Step::new(Op::PickSlot(PickSlotSpec {
                    chooser: Who::Me,
                    among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::TypeIs(crate::types::ct::METAL)),
                    msg: "CHOOSE_POKEMON_TO_ATTACH_CARDS",
                })),
                Step::new(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Discard),
                    predicate: Pred::All(&[Pred::BasicEnergy, Pred::Provides(crate::types::ct::METAL)]),
                    // "Up to 2" from a public zone takes at least 1 when played from the hand; through
                    // an attack it may be 0 (rulings 1778, 1844, 1853).
                    bounds: Bounds { min: Num::If(&Cond::ViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Min(&Num::Lit(2), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Provides(crate::types::ct::METAL)]))) },
                    into: 0,
                    soft: true,
                    msg: "CHOOSE_CARD_TO_ATTACH",
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Discard), to: ZoneRef(Who::Me, Zone::PickedSlot), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            ],
            no: &[],
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
