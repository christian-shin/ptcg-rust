//! Goldeen (TWM): Festival Lead. Whirlpool - 10; flip a coin. If heads, discard an Energy from your opponent's Active Pokémon.
//!
//! Festival Lead (with Festival Grounds in play, use an attack it has twice) is a declaration the core reads when the second
//! attack would be offered (`Modifier::AttackTwice`; batch 8 C12c). The coin is always flipped, also when the
//! Defending Pokémon has no Energy (a printed coin flip always happens, RULES.md); on heads the discard is a LeavePlay (to the discard pile) of the Energy the attacker chooses,
//! caused by this attack (D1).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Goldeen@TWM|PRE",
    // Festival Lead: if Festival Grounds is in play, this Pokémon may use an attack it has twice (read by the core when
    // the second attack would be offered: `attack::attack_twice`).
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::AttackTwice(AttackTwiceSpec { guard: Cond::StadiumInPlay(Pred::Name("Festival Grounds")) }) }],
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT }))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
