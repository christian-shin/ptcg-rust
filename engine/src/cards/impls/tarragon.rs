//! Tarragon (POR 85): put up to 4 in any combination of [F] Pokémon and Basic [F] Energy cards from your discard pile into
//! your hand.
//!
//! The chosen cards (at least 1 when played from the hand; any number, even 0, when the Supporter's effect is used by an
//! attack, id2376 and APR D-05) go from the discard pile into the hand (PutIntoHand, revealed by the move,
//! id787).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tarragon",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            // Up to 4 in any combination of [F] Pokémon and Basic [F] Energy cards from the discard pile: at
            // least 1 when played from the hand, any number through Look-Alike Show (id2376).
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::OneOf(&[Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::FIGHTING)])]),
                bounds: Bounds { min: Num::If(&Cond::TrainerViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(4) },
                caps: &[
                    Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::Lit(4), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::FIGHTING)]))) },
                    Cap { kind: CapKind::Energy, max: Num::Min(&Num::Lit(4), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]))) },
                ],
                into: 0,
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::PutIntoHand(PutIntoHandSpec { from: ZoneRef(Who::Me, Zone::Discard), cards: CardSel::Chosen(0), ..PutIntoHandSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
