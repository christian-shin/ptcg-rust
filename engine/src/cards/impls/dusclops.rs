//! Dusclops (SFA): Cursed Blast — put 5 damage counters on 1 of your
//! opponent's Pokémon, then this Pokémon is Knocked Out. Will-o-Wisp — 50.
//!
//! Twinleaf: no once-per-turn marker; the self-KO is `damage += 999` on the
//! slot holding this card (the checkState reducer then knocks it out).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dusclops",
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[
            Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::PokemonBenchFirst(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), counters: Num::Lit(5), cause: CounterCause::Effect })),
            Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::This, mode: KnockOutMode::Direct, when: Cond::True })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Cursed Blast's hand-written code, still called by Dusknoir until it is converted.
mod legacy {
    use crate::cards::prelude::*;

    /// `WAS_POWER_USED(effect, 0, this)`: ChoosePokemonPrompt on the opponent's
    /// Pokémon ([BENCH, ACTIVE], min 1, max 1, no cancel).
    pub fn cursed_blast_reduce(g: &mut Game, me: CardId, e: EffId) -> R {
        if !was_power_used(g, e, 0, me) {
            return Ok(());
        }
        let p = match *g.e(e) {
            Effect::Power { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
        Ok(())
    }

    pub fn cursed_blast_resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res], damage: i32) -> R {
        if f.stage != 1 {
            return Ok(());
        }
        let p = f.a[0] as usize;
        let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
        if let Some(t) = targets.first() {
            g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target: *t, damage, source: me })?;
        }
        for (s, c, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if c == me {
                g.st.players[p].slots[s as usize].damage += 999;
            }
        }
        Ok(())
    }
}
pub use legacy::{cursed_blast_reduce, cursed_blast_resume};
