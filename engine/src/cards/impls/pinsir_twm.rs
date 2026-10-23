//! Pinsir (TWM 3): Slow Crunch — discard all Energy from this Pokémon. At the end of your opponent's next turn, the Defending
//! Pokémon will be Knocked Out. Superpowered Horns — 100.
//!
//! Slow Crunch puts an effect on the Defending Pokémon (an ApplyEffect event: "prevent all effects of attacks" keeps it off,
//! id2341): it is gone when that Pokémon moves to the Bench or evolves (id1981), and stays when Pinsir retreats or is Knocked
//! Out (id1982: "the effect is on the Pokemon that was attacked with Slow Crunch"). At the end of the opponent's next turn it
//! is one of that player's end-of-turn effects (APR E-21); the Knock Out is by an effect, done at the state check after every
//! end-of-turn effect (id1944), Prize cards as usual. A protection that came later doesn't stop it (id1121).
use crate::spec::prelude::*;

static SLOW_CRUNCH: EndOfTurnSpec = EndOfTurnSpec { steps: &[Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::Picked, when: Cond::True }))] };

pub static SPEC: CardSpec = CardSpec {
    class: "PinsirTWM",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::AtEndOfOpponentsNextTurn(&SLOW_CRUNCH) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
