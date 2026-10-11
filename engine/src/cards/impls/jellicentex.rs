//! Jellicent ex (WHT 45 / SV11W): Oceanic Curse - as long as this Pokémon is in the Active Spot, your opponent can't play any Item
//! cards or Pokémon Tool cards from their hand. Power Press - 80+; 80 more damage if it has at least 2 extra Energy attached (in
//! addition to this attack's cost).
//!
//! The extra Energy is the Energy provided by the Active Pokémon minus the attack's cost now. The Ability is one lock over the
//! opponent's events while this Pokémon is Active (an Ability lock stops it): "Item cards from their hand" is their PlayTrainer
//! (`Use(Played) & Source(Hand) & Card(Item)`; Fossils too, id2325); "Pokémon Tool cards from their hand" is the Attach of a Tool
//! from the hand, whatever attaches it (id25, id230); a Tool put on by an effect from another zone isn't stopped.
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
            lock: LockDecl::on(
                EventPred::Any(&[PLAY_ITEM_FROM_HAND, EventPred::All(&[EventPred::Kind(EventKind::Attach), EventPred::Source(RulesZone::Hand), EventPred::Card(Pred::Tool)])]),
                "BLOCKED_BY_ABILITY",
            ),
            while_: &[LockWhile::Active],
            ability: true,
        }) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
