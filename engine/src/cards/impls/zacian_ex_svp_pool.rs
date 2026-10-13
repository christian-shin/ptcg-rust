//! Zacian ex (SVP 198): Steel Armament - 20; search your deck for a Basic [M] Energy card and attach it to this Pokémon.
//! Then, shuffle your deck. Slashing Strike - 210; during your next turn this Pokémon can't use Slashing Strike.
//!
//! Steel Armament: one Attach event of the Energy from the deck to this Pokémon (cause: this attack; none may be found),
//! then one shuffle after it (APR E-19). Slashing Strike places a lasting lock naming this attack on this Pokémon until
//! the end of your next turn (D14); it goes with the Pokémon's other effects of attacks when it moves to the Bench or
//! evolves (APR C-15).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ZacianexSVPPool",
    attacks: &[
        AttackSpec {
            index: 0,
            // Steel Armament: search your deck for a Basic [M] Energy card and attach it to this Pokémon, then shuffle.
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Attach(AttachSpec {
                        chooser: Who::Me,
                        from: ZoneRef(Who::Me, Zone::Deck),
                        predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Metal Energy")]),
                        slots: AttachSlots::ActiveOnly,
                        target: SlotPred::Any, scan: TargetScan::InPlay,
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                        same_target: false,
                        different_targets: false,
                        valid_types: &[],
                        max_per_type: 0,
                        cancel: false,
                        onto: None,
                        cards: CardSel::All,
                     different_types: false, })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn).naming(NamedAttack::This)) }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
