//! Espeon ex (PRE 34, Tera): Psych Out — 160; discard a random card from your opponent's hand. Amazez — devolve each
//! of your opponent's evolved Pokémon by shuffling the highest Stage Evolution card on it into your opponent's deck.
//!
//! Amazez is one Devolve event per evolved Pokémon by the attack's effect (APR C-13): the preventions are asked ("prevent
//! all effects of attacks" keeps the Pokémon evolved, scenario rule-C13-amethyst-mist-energy-prevents-devolve-aud-c);
//! the devolved Pokémon keeps its counters and is Knocked Out at the state check if they reach its HP; its effects end
//! (`clear_effects_evolving`). The opponent's deck is shuffled after. Psych Out's random discard is a raw move until the
//! Discard event (B7). Tera: `TERA_RULE`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Espeonex",
    attacks: &[
        // Psych Out: discard 1 random card from your opponent's hand.
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Move(MoveSpec {
                from: ZoneRef(Who::Opp, Zone::Hand),
                to: ZoneRef(Who::Opp, Zone::Discard),
                cards: CardSel::Random(Num::Lit(1)),
                ..MoveSpec::DEFAULT
            }))],
        },
        // Amazez: devolve each of your opponent's evolved Pokémon by shuffling the highest Stage
        // Evolution card into their deck.
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::ForEach(ForEachSpec {
                    over: SlotSel::Pokemon(Who::Opp),
                    body: &[Step::new(Op::Devolve(DevolveSpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Opp, Zone::Deck), chooser: None }))],
                })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
            ],
        },
    ],
    // Tera: no attack damage while Benched.
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
