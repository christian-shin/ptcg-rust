//! Frogadier (CRI, Frogadier M4): Summoning Jutsu — search your deck for up
//! to 3 Pokémon, reveal them, and put them into your hand. Then, shuffle your
//! deck. Aqua Edge — 50.
//!
//! Twinleaf (chaos-rising file): an empty deck → nothing happens (no shuffle);
//! phase 4b R7E (rulings 336, 779, 1764): a deck without a Pokémon is still
//! searched and shuffled (it used to return); ChooseCardsPrompt min 0, max min(3, Pokémon in deck), no
//! cancel; the chosen cards are revealed (fixed in phase 4b, R4: there was
//! no reveal prompt); one MOVE_CARDS per chosen card (sourceCard = the
//! attacking Pokémon); a final ShuffleDeckPrompt with no wait.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Frogadier@CRI",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Deck), Pred::Pokemon), &Num::Lit(3)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: true },
                msg: "",
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
