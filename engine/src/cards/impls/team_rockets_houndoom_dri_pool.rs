//! Team Rocket's Houndoom (DRI 38): Cruel Coal — your opponent's Active
//! Pokémon is now Burned and Confused. Scorching Fire — 120; discard an
//! Energy from this Pokémon.
//!
//! Twinleaf: Cruel Coal reduces an AddSpecialConditionsEffect [BURNED,
//! CONFUSED]. Scorching Fire: DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(1, {},
//! 1): no prompt without Energy on the Active; otherwise a non-cancellable
//! DiscardEnergyPrompt (min = min(1, available), max = min(1, available)),
//! then one DiscardCardsEffect per source slot. Shared by Galarian Obstagoon
//! and Iron Boulder ex.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsHoundoomDRIPool",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(inflict(&[SpecialCondition::Burned, SpecialCondition::Confused], Cause::Attack))] },
        AttackSpec {
            index: 1,
            // Discard an Energy from this Pokémon.
            steps: &[Step::after_damage(Op::EnergyChoice(EnergyChoiceSpec {
                how: EnergyHow::Prompt { scope: PromptScope::Active, min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, clamp: true },
                ..EnergyChoiceSpec::DEFAULT
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// The hand-written helpers other cards still call, until they are converted.
mod legacy {
    use crate::cards::prelude::*;
    use crate::effects::AtkBase;
    use crate::cards::registry::jolteonex::discard_transfers_as_effects;

    /// `DISCARD_UP_TO_X_ENERGY_FROM_THIS_POKEMON(store, state, effect, max, {}, min)`.
    /// The continuation is `resume` stage `stage` through [`discard_up_to_chosen`].
    pub fn discard_up_to_x_energy_from_this_pokemon(g: &mut Game, me: CardId, e: EffId, max: i32, min: i32, stage: u8) -> R {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        if max <= 0 {
            return Ok(());
        }
        let a = g.st.players[p].active;
        let available = g.st.slot(p, a).cards.iter().filter(|c| g.st.cdef(*c).is_energy()).count() as i32;
        if available == 0 {
            return Ok(());
        }
        let prompt_max = max.min(available);
        let prompt_min = min.max(0).min(prompt_max);
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        let filter = Filter { super_type: Some(SuperType::Energy as u8), ..Filter::none() };
        let o = MoveOpts { allow_cancel: false, min: prompt_min as u8, max: Some(prompt_max as u8), ..Default::default() };
        g.retain_fx(e);
        let mut f = CardFrame::at(stage);
        f.a[0] = p as i32;
        f.e[0] = e;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_ENERGIES_TO_DISCARD",
            PromptKind::DiscardEnergy { player_type: PlayerType::BottomPlayer, slots, filter, o },
            Cont::Card { card: me, frame: f },
        );
        Ok(())
    }

    /// The DiscardEnergyPrompt callback.
    pub fn discard_up_to_chosen(g: &mut Game, f: CardFrame, results: &[Res]) -> R {
        let p = f.a[0] as usize;
        let e = f.e[0];
        let r = (|| -> R {
            let transfers = match results.first().copied() {
                Some(Res::CardsFrom(t)) => t,
                _ => return Ok(()),
            };
            if transfers.is_empty() {
                return Ok(());
            }
            discard_transfers_as_effects(g, p, e, transfers.as_slice())
        })();
        g.release_fx(e);
        r
    }
}
pub use legacy::{discard_up_to_x_energy_from_this_pokemon, discard_up_to_chosen};
