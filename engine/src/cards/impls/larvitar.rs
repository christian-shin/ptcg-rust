//! Larvitar (JTG): Crunch — 20; flip a coin. If heads, discard an Energy
//! from your opponent's Active Pokémon. Ported so Tyranitar (JTG) can evolve
//! in check decks.
//!
//! Twinleaf: on heads with an Energy card in the Defending Pokémon's list, a
//! ChooseCardsPrompt (Energy, exactly 1) on that list; the callback reduces a
//! DiscardCardsEffect.
//!
//! Fixed (phase 4b, R2): the prompt could be cancelled, so the discard that
//! the card text makes mandatory could be skipped. The coin flip and the
//! discard came before the damage (the Defending Pokémon's Spiky Energy was
//! already gone); they now run in AfterAttackEffect (a fresh AttackEffect's
//! data).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Larvitar",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Coin(CoinSpec { before: Cond::True, heads: &[Step::new(Op::DiscardEnergy(DiscardEnergySpec { target: SlotExpr::Active(Who::Opp), selection: EnergySelection::Chosen { pred: Pred::Energy } }))], ..CoinSpec::DEFAULT })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
