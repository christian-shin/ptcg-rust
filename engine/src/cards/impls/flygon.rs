//! Flygon (PFL 53): Sandy Flapping - once during your turn, when you play this Pokémon from your hand to evolve 1 of your Pokémon, you
//! may use this Ability. You may also use this Ability if this Pokémon is in the Active Spot and is Knocked Out by damage from an
//! attack from your opponent's Pokémon. Discard the top 2 cards of your opponent's deck. Cutting Wind - 130.
//!
//! Two uses of the one Ability, each a trigger on this card's own event: the Evolve from the hand (Arven's Greedent DRI 159's form:
//! asked only when the Ability isn't blocked and the opponent's deck has a card), and the KnockOut by damage from an attack of the
//! opponent's Pokémon while this Pokémon was in the Active Spot when the damage was done (id1992; Maractus JTG 18's form), which
//! runs while it is still in play (APR D step 2) and is a "may" like the first. Each discards the top 2 cards of the opponent's deck
//! (all of them when fewer): a Discard caused by the Ability.
use crate::spec::prelude::*;

const OPP_DECK: ZoneRef = ZoneRef(Who::Opp, Zone::Deck);
const SANDY_FLAPPING: Op = Op::If(IfSpec {
    cond: Cond::All(&[Cond::Nonempty(OPP_DECK, Pred::Any), Cond::Not(&Cond::AbilityBlocked)]),
    yes: &[Step::new(Op::May(MaySpec {
        asker: Who::Me,
        when: Cond::True,
        yes: &[Step::new(Op::Discard(DiscardSpec { from: OPP_DECK, cards: CardSel::Top(Num::Lit(2)), ..DiscardSpec::DEFAULT }))],
        no: &[],
    }))],
    no: &[],
});

pub static SPEC: CardSpec = CardSpec {
    class: "Flygon",
    triggers: &[
        Trigger {
            origin: RuleSource::Ability,
            event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
            steps: &[Step::new(SANDY_FLAPPING)],
        },
        Trigger {
            origin: RuleSource::Ability,
            event: Event::On(EventPred::All(&[
                EventPred::Kind(EventKind::KnockOut),
                EventPred::This(Role::Card),
                EventPred::DamagedActive,
                EventPred::KoBy(KoBy::AttackDamage),
                EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack)])),
            ])),
            steps: &[Step::new(SANDY_FLAPPING)],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
