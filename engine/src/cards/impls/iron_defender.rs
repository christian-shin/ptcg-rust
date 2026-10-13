//! Iron Defender (MEG 118 / PBL 110): during your opponent's next turn, all of your [M] Pokémon take 30 less damage from
//! attacks from your opponent's Pokémon (after applying Weakness and Resistance). (This includes new Pokémon that come into
//! play.)
//!
//! An effect on each of your [M] Pokémon (APR B-05 names it "an effect on the Pokémon taking the damage"; batch 8 C12c, the one
//! form for "(This includes new Pokémon that come into play.)", `Op::ArmEach`, as Walrein's Frigid Fangs): a lasting "takes 30
//! less damage" on each [M] Pokémon you have in play when it resolves and on each new one that comes into play while it lasts,
//! through the opponent's next turn (step 5 of the damage calculation, after Weakness and Resistance). A Trainer's effect isn't
//! an effect of an attack: it stays when the Pokémon moves to the Bench or evolves (APR A-05, C-15). Two copies stack, 60 less
//! (JP FAQ Iron Defender; APR B-05: every effect on the Pokémon applies).
use crate::spec::prelude::*;
use crate::types::ct;

pub static SPEC: CardSpec = CardSpec {
    class: "IronDefender",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::ArmEach(ArmEachSpec { side: Who::Me, subject: SlotPred::TypeIs(ct::METAL), what: Lasting::TakesLessDamage(30) }))],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
