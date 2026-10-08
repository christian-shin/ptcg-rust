//! Cinderace (M1L): Explosiveness (setup, core `PLAY_DURING_SETUP` tag).
//! Flame Turbo — 50; search your deck for up to 3 Basic Energy cards and
//! attach them to your Benched Pokémon in any way you like, then shuffle.
//!
//! Twinleaf: the AttachEnergyPrompt (deck, Bench, min 0 / max 3, no cancel) is followed
//! immediately by a SHUFFLE_DECK (before the answer); an empty answer shuffles
//! again, otherwise a MOVE_CARDS per transfer.
//!
//! Fixed (phase 4b, W4): with an empty deck the attack threw
//! CANNOT_USE_ATTACK (unusable, no damage); it now just does its 50 damage.
//! Fixed (phase 4b, F1): with no Benched Pokémon the prompt had no target; the attack now does its
//! damage and nothing else (rulings 1790, 336).
//!
//! Spec: the shuffle comes first (Twinleaf opens its shuffle before the attach answer, which the replay's shuffle tape matches by deck size).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Cinderace",
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Cmp(Num::BenchCount(Who::Me), CmpOp::Gt, Num::Lit(0))]), yes: &[Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })), Step::new(Op::Attach(AttachSpec {
                chooser: Who::Me,
                from: ZoneRef(Who::Me, Zone::Deck),
                predicate: Pred::BasicEnergy,
                slots: AttachSlots::Bench,
                target: Pred::Any,
                scan: TargetScan::InPlay,
                bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) },
                same_target: false,
                different_targets: false,
                valid_types: &[],
                max_per_type: 0,
                cancel: false,
                route: AttachRoute::Move,
                none_shuffles: true,
             different_types: false, }))], no: &[] }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
