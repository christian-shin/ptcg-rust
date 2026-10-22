//! Pecharunt (ASC 143): Final Chain — if this Pokémon is Knocked Out by damage from an attack from your opponent's Pokémon, search
//! your deck for a card and put it into your hand. Then, shuffle your deck. Mochi Rush — 20; during your next turn, this Pokémon's
//! Mochi Rush attack does 50 more damage (before applying Weakness and Resistance).
//!
//! Final Chain is a trigger on this card's own KnockOut by damage from an opponent's attack (while it is still in play, APR D step
//! 2): a search for any card takes 1 when the deck has any (id2061, as Amulet of Hope), then the deck is shuffled (APR E-19). Mochi
//! Rush's bonus is a `NextTurnBonus` of this card's attack (as Metagross TEF 115).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PecharuntASC",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::NextTurnBonus(NextTurnBonusSpec { attack: 0, bonus: 50 }) }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[
            EventPred::Kind(EventKind::KnockOut),
            EventPred::This(Role::Card),
            EventPred::KoBy(KoBy::AttackDamage),
            EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)])),
        ])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Search(SearchSpec {
                    pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Any, bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
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
