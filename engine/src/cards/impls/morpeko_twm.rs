//! Morpeko (TWM): Snack Seek — once during your turn, look at the top card of
//! your deck; you may discard it. Pick and Stick — attach up to 2 Basic
//! Energy cards from your discard pile to your Pokémon in any way you like.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Morpeko@TWM",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Discard), Pred::BasicEnergy),
            yes: &[Step::new(Op::Attach(AttachSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::BenchActive,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                ..AttachSpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurn("SNACK_SEARCH_MARKER"),
        needs: &[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        // Today's behavior kept: the top card is always discarded (Twinleaf answers the info prompt
        // with a yes), there is no real "you may".
        steps: &[
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Top(Num::Lit(1)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Discard), ..MoveSpec::DEFAULT })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
