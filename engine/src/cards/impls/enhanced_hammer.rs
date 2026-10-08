//! Enhanced Hammer (TWM): discard a Special Energy attached to 1 of your
//! opponent's Pokémon.
//!
//! Twinleaf: Pokémon without a Special Energy in `energies` are blocked; the
//! card choice is on the target's `energies` list (energyType SPECIAL, min 1,
//! no cancel) and the energy moves from the slot to the opponent's discard.
use crate::spec::prelude::*;

const ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(SlotExpr::Picked));

pub static SPEC: CardSpec = CardSpec {
    class: "EnhancedHammer",
    // Discard a Special Energy attached to 1 of your opponent's Pokémon.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::AnySlot(SlotSel::Pokemon(Who::Opp), SlotPred::HasSpecialEnergy)],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec {
                chooser: Who::Me,
                among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::HasSpecialEnergy),
                msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS",
            })),
            Step::new(Op::Pick(PickSpec {
                from: ATTACHED,
                predicate: Pred::SpecialEnergy,
                bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                into: 0,
                msg: "CHOOSE_CARD_TO_DISCARD",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Move(MoveSpec { from: ATTACHED, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
