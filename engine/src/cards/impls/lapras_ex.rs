//! Lapras ex (SCR 32, Tera): Power Splash - 40x; this attack does 40 damage for each Energy attached to this Pokémon. Larimar Rain -
//! look at the top 20 cards of your deck and attach any number of Energy cards you find there to your Pokémon in any way you like.
//! Shuffle the other cards back into your deck.
//!
//! The Tera rule is `TERA_RULE`. Power Splash counts the Energy this Pokémon provides, in units (Cinccino ex CRI 73's Energized
//! Slap). Larimar Rain: the top 20 cards (all of the deck when fewer) are set aside; any number (0 allowed) of the Energy cards among
//! them, Basic or Special, are attached to the Active or Benched Pokémon (Attach events caused by the attack); the others go back
//! into the deck, which is shuffled (Grumpig MEG 63's form).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);
const LOOKED: ZoneRef = ZoneRef(Who::Me, Zone::Scratch(0));

pub static SPEC: CardSpec = CardSpec {
    class: "LaprasEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), &Num::Lit(40))))] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Look(LookSpec { from: DECK, cards: CardSel::Top(Num::Lit(20)), into: 0, ..LookSpec::DEFAULT })),
                Step::after_damage(Op::Attach(AttachSpec {
                    from: LOOKED,
                    predicate: Pred::Energy,
                    slots: AttachSlots::ActiveBench,
                    bounds: Bounds { min: Num::Lit(0), max: Num::CardCount(LOOKED, Pred::Energy) },
                    ..AttachSpec::DEFAULT
                })),
                Step::after_damage(Op::PutIntoDeck(PutIntoDeckSpec { from: LOOKED, cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
        },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
