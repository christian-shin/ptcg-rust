//! Farfetch'd (TWM): Impromptu Carrier — when you put this card from your
//! hand onto your Bench, you may search your deck for a Pokémon Tool and
//! attach it to this Pokémon, then shuffle. Mach Cut — 30; discard a Special
//! Energy from your opponent's Active Pokémon (fixed in R1-8: Twinleaf had no
//! handler for it; now the attacker picks one Special Energy attached to the
//! opponent's Active with a ChooseCardsPrompt (min 1, max 1, no cancel,
//! nothing when there is none) and a DiscardCardsEffect follows, like
//! Hawlucha FST's Flying Stomp).
//!
//! Twinleaf: the prompt is created during PlayPokemonEffect propagation
//! (before the card is benched); the callback finds this card's bench index
//! (0 if not found), MOVE_CARDS the Tool there and pushes it onto `tools`
//! directly (no AttachPokemonToolEffect, no max-tools check), then shuffles
//! with no trailing wait. Fixed (W1-C): MOVE_CARDS also left the Tool in the
//! slot's `cards`, so it was in two places; it is now taken out of `cards`
//! (a Tool lives in `tools` only).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Farfetchd",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Opp)), selection: EnergySelection::Chosen { pred: Pred::All(&[Pred::Energy, Pred::Not(&Pred::BasicEnergy)]) }, ..DiscardEnergySpec::DEFAULT })),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Play }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)]), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Tool, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::AttachToolToThis,
                msg: "",
                cancel: false,
                shuffle_first: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
