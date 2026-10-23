//! Teal Mask Ogerpon (DRI 26): Grass Kagura - search your deck for a Basic [G] Energy card and attach it to 1 of your Pokémon.
//! Then, shuffle your deck. Ogre's Hammer - 120; during your next turn, this Pokémon can't use Ogre's Hammer.
//!
//! Grass Kagura: with a card in the deck, one search of 0 or 1 Basic [G] Energy attached to one Pokémon of yours, then the
//! shuffle (APR E-19; Hearthflame Mask Ogerpon DRI 44). Ogre's Hammer arms the lasting lock on this Pokémon over the named attack
//! (APR C-15; Scyther TEF).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TealMaskOgerponDRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec {
                            from: ZoneRef(Who::Me, Zone::Deck),
                            predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Grass Energy")]),
                            bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                            ..PickSpec::DEFAULT
                        },
                        destination: SearchDestination::AttachToPicked,
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
