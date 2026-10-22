//! Team Rocket's Moltres ex (DRI 31): Flame Screen - 110; during your opponent's next turn, this Pokémon takes 50 less damage from
//! attacks (after applying Weakness and Resistance). Evil Incineration - discard a Team Rocket's Energy from this Pokémon. If you
//! do, discard your opponent's Active Pokémon and all attached cards.
//!
//! Flame Screen is the lasting "takes N less damage" (Seedot TEF). Evil Incineration: the Team Rocket's Energy is chosen on this
//! Pokémon and discarded (a LeavePlay of the card; this attack has no damage, so nothing waits for it); "if you do" is
//! `Cond::Done`; the Defending Pokémon and every card attached to it leave play to its owner's discard pile (one LeavePlay,
//! caused by this attack, which "prevent all effects of attacks" stops: the Pokémon stays).
use crate::spec::prelude::*;

const THIS_ENERGY: ZoneRef = ZoneRef(Who::Me, Zone::AttachedEnergy(SlotExpr::This));

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMoltresEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::TakesLessDamage(50, DamageSource::Any) }))] },
        AttackSpec {
            index: 1,
            steps: &[
                Step::after_damage(Op::Pick(PickSpec {
                    from: THIS_ENERGY,
                    predicate: Pred::Name("Team Rocket's Energy"),
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    soft: true,
                    ..PickSpec::DEFAULT
                })),
                Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::This)), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
                Step::after_damage(Op::If(IfSpec {
                    cond: Cond::Done,
                    yes: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: OPP_ACTIVE, destination: ZoneRef(Who::Opp, Zone::Discard), top: false }))],
                    no: &[],
                })),
            ],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
