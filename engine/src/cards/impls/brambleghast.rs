//! Brambleghast (PFL 47): Prison Panic — once during your turn, when you play this Pokémon from your hand to evolve 1 of your
//! Pokémon, you may use this Ability. Make your opponent's Active Pokémon Confused. Psychic Sphere — 80.
//!
//! The Ability is an `On(Evolve & This(Card) & Source(Hand))` trigger (evolving from the hand includes Rare Candy, id285, id1998;
//! as Kadabra MEG 55 and Hariyama MEG 73): a "you may" whose yes is the Special Condition event on the opponent's Active
//! Pokémon with the Ability as its cause (a Pokémon that can't be Confused keeps it out; the yes isn't asked when nothing would
//! change, user decision 2a: with the Defending Pokémon already Confused, `Cond::WouldChangeConditions` is false, APR B-01).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Brambleghast",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::Not(&Cond::AbilityBlocked),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::WouldChangeConditions(OPP_ACTIVE, &[SpecialCondition::Confused]),
                yes: &[Step::new(Op::Conditions(ConditionsSpec { target: OPP_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Confused]), when: Cond::True }))],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
