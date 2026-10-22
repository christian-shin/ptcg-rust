//! Misty's Psyduck (DRI 45): Flustered Leap - once during your turn, if this Pokémon is on your Bench, you may discard the bottom
//! card of your deck. If you do, discard all cards from this Pokémon and put this Pokémon on top of your deck. Sprinkle Water - 30.
//!
//! The Ability needs this Pokémon on the Bench (a printed condition). You may discard the bottom card of your deck (a Discard
//! event, caused by the Ability; an empty deck has none); "if you do" is `Cond::Done`: the Energy and Pokémon Tools attached to it are
//! discarded (LeavePlay events), then this Pokémon leaves play and its card goes on top of your deck (`RemoveFromPlaySpec::top`).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "MistysPsyduck",
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn,
        needs: &[Cond::Slot(SlotExpr::This, SlotPred::IsBench)],
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Discard(DiscardSpec { from: DECK, cards: CardSel::Bottom(Num::Lit(1)), ..DiscardSpec::DEFAULT })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Done,
                    yes: &[
                        Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::AttachedEnergy(SlotExpr::This)), ..DiscardSpec::DEFAULT })),
                        Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Tools(SlotExpr::This)), ..DiscardSpec::DEFAULT })),
                        Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: DECK, top: true })),
                    ],
                    no: &[],
                })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
