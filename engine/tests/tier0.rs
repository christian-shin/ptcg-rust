//! Tier 0 (PLAN.md 4.4): the generated card database equals data/cards.json
//! (the printed fields of every card), every pool row has its card, and the
//! names are the official ones.

use ptcg::carddb::{cards, def_by_full_name};
use serde_json::Value;

fn u8s(v: &Value) -> Vec<u8> {
    v.as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap() as u8).collect()).unwrap_or_default()
}

/// (card name, attack names, Ability names) of an official card text (the reading that named the cards of data/cards.json).
fn parse_official(text: &str) -> (String, Vec<String>, Vec<String>) {
    let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let (mut atk, mut abil) = (Vec::new(), Vec::new());
    let mut i = 1;
    while i < lines.len() {
        let l = lines[i];
        if l == "Ability:" && i + 1 < lines.len() {
            abil.push(lines[i + 1].to_string());
            i += 2;
            continue;
        }
        if (l == "0" || l.chars().all(|c| "GRWLPFDMCNY".contains(c))) && i + 1 < lines.len() {
            let n = lines[i + 1];
            // strip a trailing damage figure ("30", "30+", "20×"), never letters of the name
            let t = n.trim_end_matches(|c| "×x+-".contains(c) && n.len() > 0);
            let head = match t.rsplit_once(char::is_whitespace) {
                Some((h, last)) if !last.is_empty() && last.chars().all(|c| c.is_ascii_digit()) => h,
                _ => n,
            };
            atk.push(head.trim().to_string());
            i += 2;
            continue;
        }
        i += 1;
    }
    (lines[0].to_string(), atk, abil)
}

#[test]
fn card_db_matches_card_data() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/cards.json")).unwrap();
    let data: serde_json::Map<String, Value> = serde_json::from_str(&text).unwrap();
    let pool: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/pool.json")).unwrap()).unwrap();
    let official: Value = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/official_text.json")).unwrap()).unwrap();
    assert_eq!(cards().len(), data.len());
    for (key, c) in data.iter() {
        let name = key.as_str();
        let d = ptcg::carddb::def(def_by_full_name(name).expect("card in db"));
        assert_eq!(d.full_name, name);
        assert_eq!(d.name, c["name"].as_str().unwrap(), "{}", name);
        assert_eq!((d.set, d.set_number), (c["set"].as_str().unwrap(), c["setNumber"].as_str().unwrap()), "{}", name);
        assert_eq!(d.behavior, c["behavior"].as_str().unwrap(), "{}", name);
        assert_eq!(d.super_type as u64, c["superType"].as_u64().unwrap(), "{}", name);
        assert_eq!(d.retreat, u8s(&c["retreat"]).as_slice(), "{} retreat", name);
        let tags: Vec<&str> = c["tags"].as_array().unwrap().iter().map(|t| t.as_str().unwrap()).collect();
        assert_eq!(d.tag_names, tags.as_slice(), "{} tags", name);
        let atk = c["attacks"].as_array().unwrap();
        assert_eq!(d.attacks.len(), atk.len(), "{} attacks", name);
        for (a, j) in d.attacks.iter().zip(atk) {
            assert_eq!(a.name, j["name"].as_str().unwrap());
            assert_eq!(a.cost, u8s(&j["cost"]).as_slice(), "{} cost", name);
            assert_eq!(a.damage as i64, j["damage"].as_i64().unwrap(), "{} damage", name);
        }
        let pw = c["powers"].as_array().unwrap();
        assert_eq!(d.powers.len(), pw.len(), "{} powers", name);
        for (p, j) in d.powers.iter().zip(pw) {
            assert_eq!(p.name, j["name"].as_str().unwrap());
            assert_eq!(p.power_type as u64, j["powerType"].as_u64().unwrap());
            assert_eq!(p.use_when_in_play, j["useWhenInPlay"] == true);
        }
        if d.is_pokemon() {
            assert_eq!(d.hp as i64, c["hp"].as_i64().unwrap(), "{} hp", name);
            assert_eq!(d.stage as u64, c["stage"].as_u64().unwrap(), "{} stage", name);
            assert_eq!(d.card_type, u8s(&c["cardType"]).as_slice(), "{} type", name);
            assert_eq!(d.evolves_from, c["evolvesFrom"].as_str().unwrap(), "{}", name);
            assert_eq!(d.weakness.len(), c["weakness"].as_array().unwrap().len(), "{}", name);
            for (w, j) in d.weakness.iter().zip(c["weakness"].as_array().unwrap()) {
                assert_eq!(w.card_type as u64, j["type"].as_u64().unwrap());
                assert_eq!(w.value.map(|v| v as i64), j["value"].as_i64());
            }
            for (r, j) in d.resistance.iter().zip(c["resistance"].as_array().unwrap()) {
                assert_eq!(r.card_type as u64, j["type"].as_u64().unwrap());
                assert_eq!(r.value as i64, j["value"].as_i64().unwrap());
            }
        }
        if d.is_trainer() {
            assert_eq!(d.trainer_type as u64, c["trainerType"].as_u64().unwrap(), "{}", name);
        }
        if d.is_energy() {
            assert_eq!(d.energy_type as u64, c["energyType"].as_u64().unwrap(), "{}", name);
            assert_eq!(d.provides, u8s(&c["provides"]).as_slice(), "{}", name);
        }
    }
    // Every pool row: its key is a card; the official text names the card, its attacks and Abilities.
    let mut checked = 0;
    for row in &pool {
        let key = row["key"].as_str().unwrap();
        let d = ptcg::carddb::def(def_by_full_name(key).expect("pool card in db"));
        assert_eq!(d.full_name, key);
        assert_eq!(key, format!("{} {} {}", row["name"].as_str().unwrap(), row["set"].as_str().unwrap(), row["number"].as_str().unwrap()));
        assert_eq!((d.set, d.set_number), (row["set"].as_str().unwrap(), row["number"].as_str().unwrap()), "{}", key);
        let off = official[&format!("{} {}", row["set"].as_str().unwrap(), row["number"].as_str().unwrap())]["text"].as_str().unwrap();
        let (oname, oatk, oabil) = parse_official(off);
        assert_eq!(d.name, oname, "{}", key);
        assert_eq!(d.attacks.iter().map(|a| a.name.to_string()).collect::<Vec<_>>(), oatk, "{} attack names", key);
        assert_eq!(d.powers.iter().map(|a| a.name.to_string()).collect::<Vec<_>>(), oabil, "{} Ability names", key);
        checked += 1;
        if let Some(prev) = row["prev_key"].as_str() {
            assert!(def_by_full_name(prev).is_some(), "{} in db", prev);
        }
    }
    assert!(checked >= 650, "checked {}", checked);
    // ...and the non-pool pre-evolutions in data/support_cards.json.
    let support: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/support_cards.json")).unwrap()).unwrap();
    for s in &support {
        let n = s["key"].as_str().unwrap();
        assert!(def_by_full_name(n).is_some(), "{} in db", n);
    }
}
