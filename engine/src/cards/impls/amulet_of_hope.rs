//! Amulet of Hope (SSP 162, Pokémon Tool, ACE SPEC): if the Pokémon this card is attached to is Knocked Out by damage from an attack
//! from your opponent's Pokémon, search your deck for up to 3 cards and put them into your hand. Then, shuffle your deck.
//!
//! A trigger on the holder's KnockOut by damage from an opponent's attack (while it is still in play, APR D step 2): the search takes
//! 1 to 3 cards, at least 1 when the deck has any (id2061), then the deck is shuffled (APR E-19). A Tool lock turns it off.
use crate::spec::prelude::*;

const BY_OPP_ATTACK: CausePred = CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)]);

pub static SPEC: CardSpec = CardSpec {
    class: "AmuletOfHope",
    triggers: &[Trigger {
        origin: RuleSource::Tool,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::KnockOut), EventPred::Slot(SlotPred::Holder), EventPred::KoBy(KoBy::AttackDamage), EventPred::Cause(BY_OPP_ATTACK)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: false },
                    cancel: false,
                })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
