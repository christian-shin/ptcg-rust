//! Hilda (SV11W / WHT): search your deck for an Evolution Pokémon and an
//! Energy card, reveal them, put them into your hand, then shuffle.
//!
//! Twinleaf: every non-(Evolution Pokémon / Energy) deck card is blocked,
//! `max = min(evolutions,1) + min(energies,1)` with `maxPokemons` /
//! `maxEnergies`; ShowCards only when something was taken; the final
//! ShuffleDeckPrompt has no trailing wait.
//!
//! Fixed (phase 4b, R2): playable with an empty deck; it now throws
//! CANNOT_PLAY_THIS_CARD before any state change (rulings 779, 851).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hilda",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::OneOf(&[Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)]), Pred::Energy]), bounds: Bounds { min: Num::Lit(0), max: Num::Add(&Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)])), &Num::Lit(1)), &Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Energy), &Num::Lit(1))) }, caps: &[Cap { kind: CapKind::Pokemon, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::All(&[Pred::Pokemon, Pred::Not(&Pred::Basic)])), &Num::Lit(1)) }, Cap { kind: CapKind::Energy, max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Energy), &Num::Lit(1)) }], ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Still used by Dawn and Colress's Obsession until they are converted.
pub use legacy::reveal_then_shuffle_resume;

mod legacy {
    use crate::cards::prelude::*;

/// Stage 1: move to hand, ShowCards (if any) → stage 2: ShuffleDeck → stage 3: apply.
    pub fn reveal_then_shuffle_resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
        let p = f.a[0] as usize;
        let first = results.first().copied().unwrap_or(Res::Null);
        match f.stage {
            1 => {
                let cards: Vec<CardId> = first.cards().to_vec();
                move_cards(g, ListRef::Deck(p as u8), ListRef::Hand(p as u8), &cards, me)?;
                if !cards.is_empty() {
                    let mut nf = CardFrame::at(2);
                    nf.a[0] = p as i32;
                    let id = g.player_id(1 - p);
                    g.prompt(id, "CARDS_SHOWED_BY_THE_OPPONENT", PromptKind::ShowCards, Cont::Card { card: me, frame: nf });
                } else {
                    shuffle(g, me, p);
                }
                Ok(())
            }
            2 => {
                shuffle(g, me, p);
                Ok(())
            }
            3 => {
                if let Res::Order(o) = first {
                    crate::game::apply_order(&mut g.st.players[p].deck, o.as_slice());
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    
    fn shuffle(g: &mut Game, me: CardId, p: usize) {
        let mut f = CardFrame::at(3);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(id, "", PromptKind::ShuffleDeck, Cont::Card { card: me, frame: f });
    }
}
