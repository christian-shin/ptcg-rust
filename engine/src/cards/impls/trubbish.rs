//! Trubbish (M4 / CRI 56): Acid Spray — 10; flip a coin, if heads discard an
//! Energy attached to your opponent's Active Pokémon.
//!
//! Twinleaf: DISCARD_AN_ENERGY_FROM_OPPONENTS_ACTIVE_POKEMON in the coin
//! callback (no prompt when the Active has no Energy card).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Trubbish",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::Coin(CoinSpec {
            heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(OPP_ACTIVE), selection: EnergySelection::Cards { min: Num::Lit(1), max: Num::Lit(1), kind: EnergyKind::Any, cancel: false, energies_only: false }, ..DiscardEnergySpec::DEFAULT }))],
            ..CoinSpec::DEFAULT
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// The hand-written helpers other cards still call, until they are converted.
mod legacy {
    use crate::cards::prelude::*;
    use crate::effects::AtkBase;

    /// `DISCARD_AN_ENERGY_FROM_OPPONENTS_ACTIVE_POKEMON(store, state, effect)`
    /// (count 1, any type). `atk` must be retained; the continuation at `stage`
    /// (handled by [`discard_chosen`]) releases it. Releases it when no prompt
    /// opens.
    pub fn discard_an_energy_from_opponents_active(g: &mut Game, me: CardId, atk: EffId, stage: u8) -> R {
        let (p, o) = match *g.e(atk) {
            Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
            _ => {
                g.release_fx(atk);
                return Ok(());
            }
        };
        let a = g.st.players[o].active;
        let has = g.st.slot(o, a).cards.iter().any(|c| g.st.cdef(c).is_energy());
        if !has {
            g.release_fx(atk);
            return Ok(());
        }
        let mut f = CardFrame::at(stage);
        f.e[0] = atk;
        f.l[0] = o as u8;
        f.l[1] = a;
        choose_cards(g, p, "CHOOSE_CARD_TO_DISCARD", ListRef::Slot(o as u8, a), Filter::super_type(SuperType::Energy), ChooseCardsOpts::new(1, 1, false), Cont::Card { card: me, frame: f });
        Ok(())
    }

    /// Callback of the prompt opened by [`discard_an_energy_from_opponents_active`]:
    /// DiscardCardsEffect on the Active the prompt listed.
    pub fn discard_chosen(g: &mut Game, f: CardFrame, results: &[Res]) -> R {
        let atk = f.e[0];
        let cards: Vec<CardId> = results.first().copied().unwrap_or(Res::Null).cards().to_vec();
        let r = (|| -> R {
            if cards.is_empty() {
                return Ok(());
            }
            if let Effect::Attack { p, opp, attack, source, .. } = *g.e(atk) {
                let target = SlotRef::new(f.l[0] as usize, f.l[1]);
                let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target };
                let mut cs = SVec::new();
                for c in cards {
                    cs.push(c);
                }
                g.run_fx(Effect::DiscardCards { b, cards: cs })?;
            }
            Ok(())
        })();
        g.release_fx(atk);
        r
    }
}
pub use legacy::{discard_an_energy_from_opponents_active, discard_chosen};
