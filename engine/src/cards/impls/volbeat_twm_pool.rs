//! Volbeat (TWM 9): Quick Sign - if you go first, you can use this attack during your first turn. Search your deck for up
//! to 2 Basic Pokémon and put them onto your Bench. Then, shuffle your deck. Coordinated Strike - 20+; if Illumise is on
//! your Bench, 60 more damage.
//!
//! "If you go first, you can use this attack during your first turn" is `AttackFlags { first_turn: Attack(0) }` on Quick Sign,
//! read by the UseAttack's first-turn check.
//!
//! Each Pokémon goes onto the Bench through the EnterPlay event (source the deck, mode effect); an attack's "up to 2" may
//! be 0 (id2301); the deck is shuffled once (APR E-19). With an empty deck or a full Bench the attack does nothing.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "VolbeatTWMPool",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: Some(FirstTurnAttack::Attack(0)), shred: None }) }],
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::BenchSpace(Who::Me)]),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Basic, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::Lit(2), &Num::OpenBench(Who::Me)) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Bench,
                        msg: "",
                        cancel: true,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        // Coordinated Strike: 60 more damage if Illumise is on your Bench.
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(60, Cond::InPlay(Who::Me, PlayScope::Bench, Pred::Name("Illumise"))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
