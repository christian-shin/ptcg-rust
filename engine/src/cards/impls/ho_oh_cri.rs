//! Ho-Oh (CRI 10): Flames of Revival — put up to 3 Basic Pokémon from your discard pile onto your Bench. Bright Wing — 130;
//! discard a [R] Energy from this Pokémon.
//!
//! Flames of Revival is a Search of the discard pile onto the Bench, 0 to 3 Basic Pokémon: the attack is used even with no Basic
//! Pokémon in the discard pile or no room on the Bench (id2351; as Duskull SFA 40). Each one goes onto the Bench by an EnterPlay
//! event (effect mode, source the discard pile). Bright Wing discards one Energy card that provides [R] after the damage (a
//! LeavePlay caused by the attack).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "HoOhCRI",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::All(&[Pred::Pokemon, Pred::Basic]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(3) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Bench,
                cancel: false,
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: ct::FIRE, up_to: false }, ..DiscardEnergySpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
