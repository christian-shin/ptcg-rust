//! Ange Floette (CRI 75, Stadium): you can put this card into play only if you discard a Prism Tower in play, and you can put this
//! card into play during the same turn you play Prism Tower. Each Mega Floette ex in play (both yours and your opponent's) gets
//! +150 HP.
//!
//! The first clause is a printed condition of the play (`Cond::Printed`): the Stadium in play is a Prism Tower (either player's:
//! JP FAQ Prism Tower x Ange Floette). The Prism Tower is discarded by the rule that replaces the Stadium in play (APR B-04); a
//! coin-gated lock that stops the play (Seismitoad's Quaking Fist: tails) discards this card and keeps the Prism Tower (JP FAQ
//! ガマゲロゲ x プリズムタワー). "During the same turn you play Prism Tower" is a permission of the card played lifting
//! `Limit::OneStadiumPerTurn` while a Prism Tower was played from the hand this turn (JP FAQ: the player may play Ange Floette
//! after Prism Tower in the same turn). +150 HP is one `HpMod` over each Mega Floette ex; when the Stadium leaves, a Pokémon whose
//! damage reaches its HP is Knocked Out at the state check (JP FAQ Prism Tower x Mega Floette ex, as id2305).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AngeFloette",
    // It can only be played when a Prism Tower is the Stadium in play (the rule then discards it).
    play: Some(PlaySpec { kind: PlayKind::Stadium, needs: &[Cond::Printed(&Cond::StadiumInPlay(Pred::Name("Prism Tower")))], steps: &[] }),
    passives: &[
        // You can put this card into play during the same turn you play Prism Tower.
        Passive {
            origin: RuleSource::CardRule,
            modifier: Modifier::Permit(PermitSpec {
                for_: EventPred::All(&[EventPred::Kind(EventKind::PlayTrainer), EventPred::This(Role::Card)]),
                lifts: &[Limit::OneStadiumPerTurn],
                while_: &[LockWhile::Unless(&Cond::Not(&Cond::PlayedThisTurn(Who::Me, Pred::Name("Prism Tower"))))],
            }),
        },
        // Each Mega Floette ex in play gets +150 HP.
        Passive {
            origin: RuleSource::Stadium,
            modifier: Modifier::HpMod(HpModSpec { amount: Num::Lit(150), subject: SlotPred::Named("Mega Floette ex"), guard: Cond::True }),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
