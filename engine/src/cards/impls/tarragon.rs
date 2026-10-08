//! Tarragon (POR, supporter): put up to 4 in any combination of [F] Pokémon
//! and Basic [F] Energy cards from your discard pile into your hand.
//!
//! Twinleaf: DiscardToHandEffect is probed first (a prevented effect just
//! leaves the card to be discarded); the supporter check follows, then the
//! card moves to the supporter pile. Energy counts only when it is a Basic
//! card named 'Fighting Energy'; Pokémon by `pokemonHasCardType`. The
//! prompt allows 1 to 4 from the hand, 0 to 4 through Look-Alike Show (maxPokemons/maxEnergies = min(count, 4)); the cards
//! are moved to the hand first and shown to the opponent afterwards.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tarragon",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            // Up to 4 in any combination of [F] Pokémon and Basic [F] Energy cards from the discard pile: at
            // least 1 when played from the hand, any number through Look-Alike Show (ruling 1844).
            Step::new(Op::Pick(PickSpec {
                from: ZoneRef(Who::Me, Zone::Discard),
                predicate: Pred::OneOf(&[Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::FIGHTING)])]),
                bounds: Bounds { min: Num::If(&Cond::TrainerViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(4) },
                caps: &[
                    Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::Lit(4), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::FIGHTING)]))) },
                    Cap { kind: CapKind::Energy, max: Num::Min(&Num::Lit(4), &Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::All(&[Pred::BasicEnergy, Pred::Name("Fighting Energy")]))) },
                ],
                into: 0,
                msg: "CHOOSE_CARD_TO_HAND",
                ..PickSpec::DEFAULT
            })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Discard), to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), reveal: Some(Who::Opp), ..MoveSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
