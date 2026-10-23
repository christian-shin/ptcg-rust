//! Tinkatuff (MEG 97): Haphazard Hammer - once during your turn, when you play this Pokémon from your hand to evolve 1 of
//! your Pokémon, you may use this Ability. Flip a coin. If heads, discard an Energy from your opponent's Active Pokémon.
//! Light Punch - 30.
//!
//! Kadabra MEG 55's Evolve trigger (from the hand only; an evolution from the deck doesn't fire it): asked only when the
//! Defending Pokémon has an Energy to discard; the coin's heads discards one Energy chosen from it (Croconaw ASC 42's
//! Crunch).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tinkatuff",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::If(IfSpec {
            cond: Cond::All(&[Cond::AnySlot(SlotSel::One(OPP_ACTIVE), SlotPred::HasEnergy), Cond::Not(&Cond::AbilityBlocked)]),
            yes: &[Step::new(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::True,
                yes: &[Step::new(Op::Coin(CoinSpec { heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::Energy }, ..DiscardEnergySpec::DEFAULT }))], ..CoinSpec::DEFAULT }))],
                no: &[],
            }))],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
