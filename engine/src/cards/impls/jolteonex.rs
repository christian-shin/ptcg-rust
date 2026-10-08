//! Jolteon ex (PRE, Tera): Flashing Spear — 60+; you may discard up to 2
//! Basic Energy from your Benched Pokémon, 90 more damage for each card
//! discarded. Dravite — 280; during your next turn this Pokémon can't attack.
//!
//! Twinleaf: Flashing Spear resets `damage = 60`, then
//! DISCARD_UP_TO_X_ENERGY_FROM_YOUR_POKEMON(2, { energyType: BASIC }, 0,
//! [BENCH]) — no prompt without Basic Energy on the Bench; one
//! DiscardCardsEffect per source slot (first-seen order), then
//! `damage = 60 + 90 * transfers`. Dravite: THIS_POKEMON_CANNOT_ATTACK_NEXT_TURN.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Jolteonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::FromBench { max: 2, pred: Pred::BasicEnergy }, ..DiscardEnergySpec::DEFAULT })),
            Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::Last, &Num::Lit(90)), when: Cond::True })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::CannotAttackNextTurn })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Still used by Team Rocket's Houndoom until it is converted.
pub use legacy::{discard_transfers_as_effects};

mod legacy {
    use crate::cards::prelude::*;

    /// `discardTransfersAsEffects`: one DiscardCardsEffect per source slot, in
    /// first-seen order.
    pub fn discard_transfers_as_effects(g: &mut Game, p: usize, e: EffId, transfers: &[(CardTarget, CardId)]) -> R {
        let mut groups: Vec<(SlotRef, SVec<CardId, 64>)> = Vec::new();
        for (from, c) in transfers.iter() {
            let s = get_target(&g.st, p, *from)?;
            match groups.iter_mut().find(|(t, _)| *t == s) {
                Some((_, v)) => v.push(*c),
                None => {
                    let mut v = SVec::new();
                    v.push(*c);
                    groups.push((s, v));
                }
            }
        }
        let (opp, attack, source) = match *g.e(e) {
            Effect::Attack { opp, attack, source, .. } => (opp, attack, source),
            _ => return Ok(()),
        };
        for (target, cards) in groups {
            let b = AtkBase { attack_effect: e, player: p as u8, opponent: opp, attack, source, target };
            g.run_fx(Effect::DiscardCards { b, cards })?;
        }
        Ok(())
    }
}
