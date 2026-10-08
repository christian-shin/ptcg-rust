//! Mega Kangaskhan ex (M1S, Basic Mega ex): Run Errand - once during your
//! turn, if this Pokémon is Active, draw 2 cards (1 Run Errand per turn).
//! Rapid-Fire Combo - 200+, flip until tails, 50 more per heads.
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
            Cond::Not(&Cond::HasMarker { who: Who::Me, name: "RUN_ERRAND_USED_MARKER", from: MarkerFrom::Any }),
            Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
        ],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) })),
            Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "RUN_ERRAND_USED_MARKER", source: RuleSource::CardRule })),
        ],
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "RUN_ERRAND_USED_MARKER", from: MarkerFrom::Any }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
