//! Mega Kangaskhan ex (MEG 104): Run Errand - once during your turn, if this Pokémon is in the Active Spot, you may use
//! this Ability: draw 2 cards; you can't use more than 1 Run Errand Ability each turn. Rapid-Fire Combo - 200+; flip a
//! coin until you get tails; 50 more damage for each heads.
//!
//! Run Errand is one Draw event of 2 (cause: this Ability); the usage record is the player's (any copy) and ends with
//! the owner's turn. Rapid-Fire Combo flips before the damage and adds 50 per heads.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaKangaskhanex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::Coin(CoinSpec { flips: Flips::UntilTails, per_heads: PerHeads::DamageAdd(50), ..CoinSpec::DEFAULT }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        // One Run Errand per turn for the player, whichever copy used it.
        once: Once::No,
        needs: &[
            Cond::IsActive(SlotExpr::This),
            Cond::Not(&Cond::Used(UsedSpec::ability(0, UsedBy::Player))),
        ],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            Step::new(Op::Used(UsedSpec::ability(0, UsedBy::Player))),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
