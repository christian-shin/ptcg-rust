//! Grumpig (MEG 63): Energized Steps - once during your turn, when you play this Pokémon from your hand to evolve 1 of your Pokémon,
//! you may use this Ability. Look at the top 4 cards of your deck and attach any number of Basic Energy cards you find there to your
//! Pokémon in any way you like. Shuffle the other cards back into your deck. Psychic Sphere - 60.
//!
//! An Evolve-from-hand trigger (Arven's Greedent DRI 159's form: asked when the Ability isn't blocked and the deck has a card). The top
//! 4 cards are set aside; any number (0 allowed) of the Basic Energy cards among them are attached to the Active or Benched
//! Pokémon (Attach events caused by the Ability); the others go back into the deck, which is shuffled (Metang TEF 105 and Waitress).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const LOOKED: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(0));

pub static SPEC: CardSpec = CardSpec {
    class: "Grumpig",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::All(&[Cond::Nonempty(DECK, Pred::Any), Cond::Not(&Cond::AbilityBlocked)]),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                yes: &[
                    Step::new(Op::Look(LookSpec { from: DECK, cards: CardSel::Top(Num::Lit(4)), into: 0, ..LookSpec::DEFAULT })),
                    Step::new(Op::Attach(AttachSpec {
                        from: LOOKED,
                        predicate: Pred::BasicEnergy,
                        slots: AttachSlots::ActiveBench,
                        bounds: Bounds { min: Num::Lit(0), max: Num::CardCount(LOOKED, Pred::BasicEnergy) },
                        ..AttachSpec::DEFAULT
                    })),
                    Step::new(Op::PutIntoDeck(PutIntoDeckSpec { from: LOOKED, cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
                ],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
