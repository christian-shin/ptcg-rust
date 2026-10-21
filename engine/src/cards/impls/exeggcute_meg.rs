//! Exeggcute (MEG 4): Jam-Packed - search your deck for a Basic [G] Energy card and attach it to this Pokémon. Then, shuffle
//! your deck.
//!
//! The Energy is chosen from the deck (none may be found), one Attach event of it onto this Pokémon, then one shuffle (APR E-19).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ExeggcuteMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                    into: 0,
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::If(IfSpec {
                    cond: Cond::Chosen(0),
                    yes: &[Step::new(Op::Attach(AttachSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Chosen(0), onto: Some(SlotExpr::This), ..AttachSpec::DEFAULT }))],
                    no: &[],
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
