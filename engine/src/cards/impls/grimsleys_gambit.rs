//! Grimsley's Move (PFL / M2, class GrimsleysGambit): look at the top 7
//! cards of your deck and put a [D] Pokémon you find there onto your Bench;
//! shuffle the other cards and put them on the bottom of your deck. Can't be
//! used on your first turn.
//!
//! Fixed (phase 4b): at least 1 when a [D] Pokémon is among the 7 looked-at cards
//! and the card is played from the hand (rulings 1778/1853; 0 through Look-Alike Show).
//!
//! Twinleaf: no move to the supporter pile and no preventDefault (the core
//! discards the Supporter normally); checks in order: Supporter played, empty
//! deck, full Bench, turn 1/2. The other cards are shuffled with Chance.shuffle before they go to
//! the bottom (aud-d fix; Twinleaf's ShuffleDeckPrompt used to be applied to nothing). The Supporter pile → discard move
//! runs before the prompt answers, and the new Pokémon gets
//! `pokemonPlayedTurn = turn` (no PlayPokemonEffect).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "GrimsleysGambit",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Cmp(Num::Turn, CmpOp::Gt, Num::Lit(2)), Cond::BenchSpace(Who::Me)],
        steps: &[
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Top(Num::Lit(7)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::DARK)]), bounds: Bounds { min: Num::If(&Cond::All(&[Cond::Not(&Cond::TrainerViaAttack), Cond::Nonempty(ZoneRef(Who::Me, Zone::Scratch(0)), Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::DARK)]))]), &Num::Lit(1), &Num::Lit(0)), max: Num::Lit(1) }, into: 1, msg: "CHOOSE_CARD_TO_PUT_ONTO_BENCH", ..PickSpec::DEFAULT })),
            Step::new(Op::PlayFromZone(PlayFromZoneSpec { cards: 1, who: Who::Me })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, shuffle_first: true, ..MoveSpec::DEFAULT })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
