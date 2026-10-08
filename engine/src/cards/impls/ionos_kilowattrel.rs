//! Iono's Kilowattrel (JTG / ASC): Flashing Draw - you must discard a Basic
//! [L] Energy from this Pokémon to use this Ability; once during your turn,
//! draw cards until you have 6 cards in your hand. Mach Bolt - 70.
//!
//! Twinleaf quirks kept: the player marker RUMBLING_ENGINE_MARKER (source
//! this card) is cleared on this card's PlayPokemonEffect and on *every*
//! EndTurnEffect (whoever's turn it is, for that effect's player). The
//! Ability throws at 6+ cards in hand or with no cards in the deck (phase 4b,
//! ruling n=1633: the effect is to draw cards), when already used, or without a
//! "Lightning Energy" named basic Energy on the Pokémon. With exactly one
//! such Energy it is discarded without a prompt; otherwise a ChooseCards
//! prompt over the slot (min 0, max 1, cancellable; an empty answer does
//! nothing). Draw is a plain `deck.moveTo(hand, n)`.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IonosKilowattrel",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[Cond::Cmp(Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)), CmpOp::Lt, Num::Lit(6)), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Not(&Cond::HasMarker { who: Who::Me, name: "RUMBLING_ENGINE_MARKER", from: MarkerFrom::This }), Cond::AnySlot(SlotSel::One(SlotExpr::This), SlotPred::AnyCard(Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")])))],
        steps: &[
            Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::This, selection: EnergySelection::CostOne { pred: Pred::All(&[Pred::BasicEnergy, Pred::Name("Lightning Energy")]) } })),
            Step::new(Op::If(IfSpec { cond: Cond::Cmp(Num::Last, CmpOp::Gt, Num::Lit(0)), yes: &[Step::new(Op::UseAbility(UseAbilitySpec { marker: "RUMBLING_ENGINE_MARKER" })), Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::UntilHandSize(Num::Lit(6)) }))], no: &[] })),
        ],
    }],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEndTurn(OnEndTurnSpec { whose: Turn::Owner }), steps: &[Step::new(Op::ClearMarker(ClearMarkerSpec { scope: MarkerScope::Player(Who::Me), name: "RUMBLING_ENGINE_MARKER", from: MarkerFrom::This }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
