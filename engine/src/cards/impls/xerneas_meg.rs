//! Xerneas (MEG 64): Geo Gate - search your deck for up to 3 Basic [P] Pokémon and put them onto your Bench. Then, shuffle your
//! deck. Bright Horns - 120; during your next turn, this Pokémon can't use Bright Horns.
//!
//! Geo Gate: with room on the Bench, one Search of 0 to 3 Basic [P] Pokémon onto the Bench (at most the open spaces, an EnterPlay
//! each), then one shuffle (APR E-19; Swinub JTG 77). Bright Horns arms the lasting lock on this Pokémon over the named attack
//! (APR C-15; Scyther TEF).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "XerneasMEG",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::BenchSpace(Who::Me),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            predicate: Pred::All(&[Pred::Basic, Pred::PokemonType(ct::PSYCHIC)]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                            ..PickSpec::DEFAULT
                        },
                        destination: SearchDestination::Bench,
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
