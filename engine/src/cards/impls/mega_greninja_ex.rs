//! Mega Greninja ex (CRI / M4): Mortal Shuriken — once during your turn, if
//! this Pokémon is Active, you may discard a Basic [W] Energy card from your
//! hand; place 6 damage counters on 1 of your opponent's Pokémon. Ninja
//! Spinner — 120; you may put a [W] Energy attached to this Pokémon into your
//! hand for 80 more damage.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaGreninjaex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::before_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::True,
            msg: "WANT_TO_USE_EFFECT_OF_ATTACK",
            // The 80 more damage is done whether or not a [W] Energy could be taken (ruling 1822).
            yes: &[
                Step::new(more_damage_if(80, Cond::True)),
                Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(MY_ACTIVE), selection: EnergySelection::ChooseToHand { count: 1, ty: crate::types::ct::WATER, up_to: false }, ..DiscardEnergySpec::DEFAULT })),
            ],
            no: &[],
        }))],
    }],
    powers: &[PowerSpec {
        index: 0,
        // The use is only recorded once the target is chosen (cancelling the discard uses nothing).
        once: Once::No,
        needs: &[
            Cond::IsActive(SlotExpr::This),
            Cond::Not(&Cond::HasMarker { who: Who::Me, name: "MORTAL_SHURIKEN_MARKER", from: MarkerFrom::This }),
            Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::All(&[Pred::BasicEnergy, Pred::Provides(crate::types::ct::WATER)])),
            Cond::AnySlot(SlotSel::Pokemon(Who::Opp), SlotPred::Any),
        ],
        steps: &[
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Hand), predicate: Pred::All(&[Pred::BasicEnergy, Pred::Name("Water Energy")]), bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) }, cancel: true, soft: true, into: 0, msg: "CHOOSE_CARD_TO_DISCARD", ..PickSpec::DEFAULT })),
            Step::new(Op::If(IfSpec {
                cond: Cond::Chosen(0),
                yes: &[
                    Step::new(Op::Discard(DiscardSpec { from: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(0), ..DiscardSpec::DEFAULT })),
                    Step::new(Op::PlaceCounters(PlaceCountersSpec {
                        target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                        counters: Num::Lit(6)
                    })),
                    Step::new(Op::SetMarker(SetMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "MORTAL_SHURIKEN_MARKER", source: RuleSource::Ability })),
                ],
                no: &[],
            })),
        ],
    }],
    triggers: &[Trigger {
        origin: RuleSource::CardRule,
        event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }),
        steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "MORTAL_SHURIKEN_MARKER", from: MarkerFrom::This }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
