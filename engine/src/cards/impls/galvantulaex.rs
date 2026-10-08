//! Galvantula ex (SCR, Tera): Charged Web — 110+, 110 more if your
//! opponent's Active Pokémon is a Pokémon ex or Pokémon V. Fulgurite — 180;
//! discard all Energy from this Pokémon; during your opponent's next turn
//! they can't play Item cards.
//!
//! Twinleaf: the V check covers V / VSTAR / VMAX tags (not V-UNION).
//! Fulgurite: CheckProvidedEnergyEffect(player) on the Active, a
//! DiscardCardsEffect of the map's cards on the Active, then
//! OPPONENT_CANNOT_PLAY_ITEM_CARDS (PlayLockEffect). Tera bench protection.
//!
//! Fixed (phase 4b, R2): the Energy was discarded in the attack handler,
//! before the damage (Voltaic Lightning Energy's +20 was lost); it is now
//! discarded in AfterAttackEffect. The Item lock stays in the attack handler.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Galvantulaex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(110, Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::OneOf(&[SlotPred::Tag(tag::POKEMON_V), SlotPred::Tag(tag::POKEMON_VSTAR), SlotPred::Tag(tag::POKEMON_VMAX), SlotPred::Tag(tag::POKEMON_EX_LOWER)])))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(Locked::Item) })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
