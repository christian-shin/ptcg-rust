//! Jellicent ex (WHT / SV11W): Oceanic Curse - while this Pokémon is your
//! Active, your opponent can't play Item cards or attach Pokémon Tools.
//! Power Press - 80+; 80 more with at least 2 extra Energy.
//!
//! Twinleaf: the extra Energy sums the provided Energy of the Active minus the
//! checked attack cost. The lock throws BLOCKED_BY_ABILITY when the
//! ability probe for the *opponent of the player* passes (i.e. the Ability is not blocked).
//!
//! Events batch 3: "Pokémon Tool cards from their hand" is the Attach event of a Tool from the hand,
//! whatever attaches it (id25, id230); a Tool put on by an effect from another zone isn't stopped. Items
//! keep the action form until PlayTrainer (batch 7).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Jellicentex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(80, Cond::Cmp(Num::Sub(&Num::EnergyOn(SlotSel::One(SlotExpr::Active(Who::Me)), EnergyUnit::ProvidedUnits), &Num::CostNow), CmpOp::Ge, Num::Lit(2)))),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec {
            binds: Binds::Opponent,
            lock: LockDecl {
                actions: &[LockedAction::PlayItem],
                card: Pred::Any,
                except: Pred::False,
                error: "BLOCKED_BY_ABILITY",
                forbids: EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::Source(RulesZone::Hand), EventPred::Card(Pred::Tool)]),
            },
            while_: &[LockWhile::Active],
            ability: true,
        }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
