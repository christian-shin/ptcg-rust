//! Mega Greninja ex (CRI / M4): Mortal Shuriken — once during your turn, if
//! this Pokémon is Active, you may discard a Basic [W] Energy card from your
//! hand; place 6 damage counters on 1 of your opponent's Pokémon. Ninja
//! Spinner — 120; you may put a [W] Energy attached to this Pokémon into your
//! hand for 80 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaGreninjaEx",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::True,
            // The 80 more damage is done whether or not a [W] Energy could be taken (id2352).
            yes: &[
                Step::new(more_damage_if(80, Cond::True)),
                Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::Choose { count: 1, ty: crate::types::ct::WATER, up_to: false }, to: EnergyDest::Hand, ..DiscardEnergySpec::DEFAULT })),
            ],
            no: &[],
        }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        // The Basic [W] Energy card is the cost of using it (APR E-30; as for N's Zoroark ex's Trade, id2115): chosen with
        // no cancel once the Ability is used, the usage record made at the use (id127, id249).
        once: Once::PerTurn,
        // "If this Pokémon is in the Active Spot" is printed; a Basic [W] Energy card in the hand is the cost's own gate (the
        // choice below).
        needs: &[
            Cond::Printed(&Cond::IsActive(SlotExpr::This)),
            Cond::AnySlot(SlotSel::Pokemon(Who::Opp), SlotPred::Any),
        ],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Water Energy")]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, into: 0, ..PickSpec::DEFAULT })),
            Step::cost(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
            Step::new(Op::PlaceCounters(PlaceCountersSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), }),
                counters: Num::Lit(6)
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
