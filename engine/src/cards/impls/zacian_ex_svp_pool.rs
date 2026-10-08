//! Zacian ex (SVP 198): Steel Armament — 20; search your deck for a Basic
//! [M] Energy card and attach it to this Pokémon, then shuffle. Slashing
//! Strike — 210; during your next turn this Pokémon can't use Slashing Strike.
//!
//! Twinleaf: ATTACH_UP_TO_X_ENERGY_FROM_DECK_TO_Y_OF_YOUR_POKEMON(1, 1, {
//! destinationSlots: [ACTIVE], energyFilter: { energyType: BASIC, name:
//! 'Metal Energy' } }): no prompt on an empty deck; a non-cancellable
//! AttachEnergyPrompt on the deck (min 0, max 1); every transfer is an
//! AttachEnergyEffect, whose reducer only moves cards out of the HAND. A card
//! chosen from the deck therefore stays in the deck while being listed in the
//! slot's `energies` (Twinleaf bug kept). Then SHUFFLE_DECK.
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
                        target: Pred::Any,
                        scan: TargetScan::InPlay,
                        bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                        same_target: false,
                        different_targets: false,
                        valid_types: &[],
                        max_per_type: 0,
                        cancel: false,
                        // Twinleaf quirk kept: an AttachEnergyEffect only moves cards out of the hand, so the card stays in the deck.
                        route: AttachRoute::Effect,
                        none_shuffles: false,
                     different_types: false, })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotUseThisAttackNextTurn }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
