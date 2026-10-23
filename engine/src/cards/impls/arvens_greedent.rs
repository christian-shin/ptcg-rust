//! Arven's Greedent (DRI 159): Greedy Order - when you play this Pokémon from your hand to evolve 1 of your Pokémon during your
//! turn, you may put up to 2 Arven's Sandwich cards from your discard pile into your hand. Rolling Tackle - 50.
//!
//! An `On(Evolve & This(Card) & Source(Hand))` trigger (evolving from the hand includes Rare Candy, id285, id1998; Kadabra MEG 55,
//! Hop's Dubwool): offered only when the Ability isn't blocked and the discard pile holds an Arven's Sandwich; a "you may" whose
//! yes is one search of 1 or 2 of them from the discard pile into the hand (a PutIntoHand event, this Ability its cause; a card
//! taken from a public zone is not revealed by the text).
use crate::spec::prelude::*;

const DISCARD: ZoneRef = ZoneRef(Who::Me, Zone::Discard);
const SANDWICH: Pred = Pred::Name("Arven's Sandwich");

pub static SPEC: CardSpec = CardSpec {
    class: "ArvensGreedent",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Nonempty(DISCARD, SANDWICH), Cond::Not(&Cond::AbilityBlocked)]),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                yes: &[Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: DISCARD, predicate: SANDWICH, bounds: Bounds { min: Num::Lit(1), max: Num::Min(&Num::CardCount(DISCARD, SANDWICH), &Num::Lit(2)) }, ..PickSpec::DEFAULT },
                    destination: SearchDestination::Hand { reveal: false },
                    cancel: false,
                }))],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
