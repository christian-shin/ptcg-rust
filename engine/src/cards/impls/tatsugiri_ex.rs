//! Tatsugiri ex (SSP 142, Tera): Surprise Pump - 100; this attack's damage isn't affected by any effects on your opponent's
//! Active Pokémon. Cinnabar Lure - look at the top 10 cards of your deck. You may put any number of Pokémon you find there onto
//! your Bench. Shuffle the other cards back into your deck. Tera: as long as this Pokémon is on your Bench, prevent all damage
//! done to it by attacks.
//!
//! The Tera rule is `TERA_RULE`, a card rule that Ability locks don't touch. Surprise Pump sets the IgnoreDefenderEffects flag
//! (Shred: Druddigon WHT 71). Cinnabar Lure sets the top 10 cards aside (`Op::Look`), puts 0 to as many Pokémon as there are open
//! Bench spaces onto the Bench from there (Grimsley's Move), and the other cards go back into the deck, which is then shuffled once
//! (Tatsugiri TWM 131).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TatsugiriEx",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Look(LookSpec { from: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::Top(Num::Lit(10)), into: 0, ..LookSpec::DEFAULT })),
                Step::after_damage(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Scratch(0)),
                    predicate: Pred::Pokemon,
                    bounds: Bounds { min: Num::Lit(0), max: Num::OpenBench(Who::Me) },
                    into: 1,
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::PlayFromZone(PlayFromZoneSpec { cards: 1, who: Who::Me })),
                Step::after_damage(Op::PutIntoDeck(PutIntoDeckSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::All, position: DeckPosition::Bottom, ..PutIntoDeckSpec::DEFAULT })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
