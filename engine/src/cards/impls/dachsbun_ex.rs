//! Dachsbun ex (SCR 67): Time to Chow Down - when you play this Pokémon from your hand to evolve 1 of your Pokémon during your turn,
//! you may heal all damage from each of your Evolution Pokémon. If you healed any damage in this way, discard all Energy from those
//! Pokémon. Wonder Shine - 130; your opponent's Active Pokémon is now Confused.
//!
//! An `On(Evolve & This(Card) & Source(Hand))` trigger (evolving from the hand includes Rare Candy, id285, id1998; Kadabra MEG 55):
//! a "you may" asked when the Ability isn't blocked and one of your Evolution Pokémon is damaged. On yes every damaged Evolution
//! Pokémon, this one included (id2016: the Fidough that evolved is healed too), is healed of all its damage (a RemoveCounters event
//! each; the loop goes over every Pokémon you have, a list that doesn't change while it runs) and the Energy of each one healed
//! is discarded (id2015: "heal all of your Evolution Pokémon and discard Energy from all of them that were healed"; the choice of
//! which to heal isn't offered).
use crate::spec::prelude::*;

const EVOLUTION: Cond = Cond::Slot(SlotExpr::Picked, SlotPred::Evolution);

pub static SPEC: CardSpec = CardSpec {
    class: "DachsbunEx",
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::AnySlot(SlotSel::Pokemon(Who::Me), SlotPred::All(&[SlotPred::Evolution, SlotPred::Damaged]))]),
            yes: &[Step::new(Op::ForEach(ForEachSpec {
                over: SlotSel::Pokemon(Who::Me),
                body: &[
                    // All the damage of an Evolution Pokémon (nothing for a Basic Pokémon, whose Heal removes none).
                    Step::new(Op::Heal(HealSpec { target: SlotTarget::Slot(SlotExpr::Picked), hp: Num::If(&EVOLUTION, &Num::DamageOn(SlotExpr::Picked), &Num::Lit(0)), clear_conditions: false })),
                    Step::new(Op::If(IfSpec {
                        cond: Cond::Done,
                        yes: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Picked), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT }))],
                        no: &[],
                    })),
                ],
            }))],
            no: &[],
        }))],
    }],
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Confused]))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
