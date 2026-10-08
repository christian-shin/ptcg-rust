//! Slither Wing (SFA): Iron Buster — 20+; 120 more if your opponent has a
//! Future Pokémon in play. Smashing Wings — 130, discard 2 Energy from this
//! Pokémon.
//!
//! DISCARD_X_ENERGY_FROM_THIS_POKEMON(2): ChooseEnergyPrompt over the
//! Active's CheckProvidedEnergy map for [C][C] (no cancel), then a
//! DiscardCardsEffect aimed at the attacker's Active.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "SlitherWing",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::before_damage(more_damage_if(120, Cond::InPlay(Who::Opp, PlayScope::All, Pred::Tag(tag::FUTURE))))] },
        AttackSpec {
            index: 1,
            // Discard 2 Energy from this Pokémon (priced as [C][C]).
            steps: &[Step::after_damage(Op::EnergyChoice(EnergyChoiceSpec { how: EnergyHow::Cost { n: Num::Lit(2), ty: ct::COLORLESS }, ..EnergyChoiceSpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// The hand-written helpers other cards still call, until they are converted.
mod legacy {
    use crate::cards::prelude::*;
    use crate::effects::AtkBase;

    /// DISCARD_X_ENERGY_FROM_THIS_POKEMON(store, state, effect, amount): resume
    /// `stage` with the chosen energy through [`discard_energy_chosen`].
    pub fn discard_x_energy_from_this_pokemon(g: &mut Game, me: CardId, e: EffId, amount: usize, stage: u8) -> R {
        discard_x_typed_energy_from_this_pokemon(g, me, e, amount, ct::COLORLESS, stage)
    }

    /// DISCARD_X_ENERGY_FROM_THIS_POKEMON with `type` (default colorless): the cost is `amount` Energy of `ty`.
    /// Ruling 1652: the payment counts Energy units and uses no more cards than `amount` (check_energy_payment).
    pub fn discard_x_typed_energy_from_this_pokemon(g: &mut Game, me: CardId, e: EffId, amount: usize, ty: CardType, stage: u8) -> R {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let a = g.st.players[p].active;
        let (pe, _) = g.run_fx(Effect::CheckProvidedEnergy { p: p as u8, source: SlotRef::new(p, a), energy_map: SVec::new() })?;
        let energy = match pe {
            Effect::CheckProvidedEnergy { energy_map, .. } => energy_map,
            _ => SVec::new(),
        };
        let mut cost = SVec::new();
        for _ in 0..amount {
            cost.push(ty);
        }
        g.retain_fx(e);
        let mut f = CardFrame::at(stage);
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(id, "CHOOSE_ENERGIES_TO_DISCARD", PromptKind::ChooseEnergy { energy, cost, allow_cancel: false }, Cont::Card { card: me, frame: f });
        Ok(())
    }

    /// Whether the attacker's Active has any Energy card attached (the guard Twinleaf cards put before
    /// DISCARD_X_ENERGY_FROM_THIS_POKEMON so that no prompt is queued without Energy).
    pub fn energy_on_active(g: &Game, e: EffId) -> bool {
        match *g.e(e) {
            Effect::Attack { p, .. } => {
                let a = g.st.players[p as usize].active;
                g.st.slot(p as usize, a).cards.iter().any(|c| g.st.cdef(c).is_energy())
            }
            _ => false,
        }
    }

    /// The ChooseEnergyPrompt callback: DiscardCardsEffect on `player.active`.
    pub fn discard_energy_chosen(g: &mut Game, f: CardFrame, results: &[Res]) -> R {
        let atk = f.e[0];
        let first = results.first().copied().unwrap_or(Res::Null);
        let r = (|| -> R {
            let mut cards = SVec::new();
            if let Res::Energy(c) = first {
                for x in c.iter() {
                    cards.push(*x);
                }
            }
            if let Effect::Attack { p, opp, attack, source, .. } = *g.e(atk) {
                let pp = p as usize;
                let target = SlotRef::new(pp, g.st.players[pp].active);
                g.run_fx(Effect::DiscardCards { b: AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target }, cards })?;
            }
            Ok(())
        })();
        g.release_fx(atk);
        r
    }
}
pub use legacy::{discard_x_energy_from_this_pokemon, discard_x_typed_energy_from_this_pokemon, energy_on_active, discard_energy_chosen};
