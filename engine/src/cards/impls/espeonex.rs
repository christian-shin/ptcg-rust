//! Espeon ex (PRE, Tera): Psych Out - 160; discard 1 random card from your
//! opponent's hand. Amethyst - devolve each of your opponent's evolved
//! Pokémon by shuffling the highest Stage Evolution card into their deck.
//! Tera: no attack damage while on the Bench.
//!
//! Twinleaf: the random discard uses `Chance.index`; the ShuffleDeckPrompt
//! has no trailing wait.
//!
//! Fixed (phase 4b, W4): Amethyst's ShuffleDeckPrompt belonged to the
//! attacking player and its order was applied to the attacker's deck, leaving
//! the opponent's deck (which received the Evolution cards) unshuffled; it now
//! shuffles the opponent's deck. Resistance is Fighting -30 (was -20).
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
        // Amethyst: devolve each of your opponent's evolved Pokémon by shuffling the highest Stage
        // Evolution card into their deck.
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::ForEach(ForEachSpec {
                    over: SlotSel::Pokemon(Who::Opp),
                    body: &[Step::new(Op::Devolve(DevolveSpec { slot: SlotExpr::Picked, destination: ZoneRef(Who::Opp, Zone::Deck) }))],
                })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck), wait: true })),
            ],
        },
    ],
    // Tera: no attack damage while Benched.
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
