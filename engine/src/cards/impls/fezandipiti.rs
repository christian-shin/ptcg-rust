//! Fezandipiti (PRE 45 / TWM 96): Adrena-Pheromone — if this Pokémon has any [D] Energy attached and is damaged by an
//! attack, flip a coin; if heads, prevent that damage. Energy Feather — 30 damage for each Energy attached to this
//! Pokémon.
//!
//! Adrena-Pheromone is a `Prevent` over `Kind(Damage) & Cause(Kind(Attack))` with a coin (`PreventSpec::on_coin`,
//! decision D8): at step 6 of the damage calculation, after the hard preventions, and only for damage there is (nothing
//! is flipped for 0 damage or damage already prevented). "Damaged by an attack" names no owner, so it covers every
//! attack's damage, its own player's included (official JP FAQ, キチキギス / サンドパン: 「自分のベンチに特性
//! 「アドレナフェロモン」がはたらいているキチキギスがいるとき、自分のサンドパンのワザ「じしん」を使った場合、…コインを
//! 投げますか？」「はい、投げます。このとき、投げたコインがオモテなら、キチキギスはダメージを受けません。」 = with your
//! Benched Fezandipiti, your own Sandslash's Earthquake: yes, you flip, and on heads Fezandipiti takes no damage). It
//! needs the Ability working and a [D] Energy unit provided on this Pokémon (a Legacy Energy counts, id1969), and the
//! flip belongs to the Pokémon's owner. Energy Feather counts every Energy unit provided on this Pokémon.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Fezandipiti",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(damage_is(Num::Mul(&Num::EnergyOn(SlotSel::One(SlotExpr::This), EnergyUnit::ProvidedUnits), &Num::Lit(30)))),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on_coin(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon, SlotPred::Provides(ct::DARK)]), DAMAGE_BY_ATTACKS)) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
