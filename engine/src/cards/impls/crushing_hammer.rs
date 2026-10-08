//! Crushing Hammer (SVI): flip a coin; if heads, discard an Energy attached
//! to 1 of your opponent's Pokémon.
use crate::spec::prelude::*;

const TARGETS: SlotSel = SlotSel::Pokemon(Who::Opp);
const ATTACHED: ZoneRef = ZoneRef(Who::Opp, Zone::AttachedEnergy(SlotExpr::Picked));

pub static SPEC: CardSpec = CardSpec {
    class: "CrushingHammer",
    // Flip a coin; if heads, discard an Energy attached to 1 of your opponent's Pokémon.
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[Cond::AnySlot(TARGETS, SlotPred::HasEnergy)],
        steps: &[Step::new(Op::Coin(CoinSpec {
            heads: &[
                Step::new(Op::PickSlot(PickSlotSpec {
                    chooser: Who::Me,
                    among: SlotSel::Filtered(&TARGETS, SlotPred::HasEnergy),
                    msg: "CHOOSE_POKEMON_TO_DISCARD_CARDS",
                })),
                Step::new(Op::Pick(PickSpec {
                    from: ATTACHED,
                    predicate: Pred::Energy,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    msg: "CHOOSE_CARD_TO_DISCARD",
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Move(MoveSpec { from: ATTACHED, to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Chosen(0), ..MoveSpec::DEFAULT })),
            ],
            ..CoinSpec::DEFAULT
        }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
