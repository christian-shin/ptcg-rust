//! Farfetch'd (TWM 132): Impromptu Carrier — when you play this Pokémon from your hand onto your Bench during your turn, you may
//! search your deck for a Pokémon Tool card and attach it to this Pokémon. Then, shuffle your deck. Mach Cut — 30; discard a
//! Special Energy from your opponent's Active Pokémon.
//!
//! Impromptu Carrier is a trigger on this card's EnterPlay from the hand to the Bench by the rules; with the Ability
//! working and a card in the deck, one search (0 or 1 Tool, a search for a kind) attaches the Tool to this Pokémon, then the
//! deck is shuffled (APR E-19). Mach Cut: the attacker chooses 1 Special Energy attached to the Defending Pokémon and it is
//! discarded after the damage (a LeavePlay caused by the attack); with none nothing happens.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Farfetchd",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]) }, ..DiscardEnergySpec::DEFAULT })),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::EnterPlay), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand), EventPred::Mode(EnterMode::Rule), EventPred::Slot(SlotPred::IsBench)])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)]), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Tool, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::AttachToolToThis,
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
