//! Tier 0 (PLAN.md 4.4): the generated card database equals the printed
//! fields of every Twinleaf class instance in the frozen pool.

use ptcg::carddb::{cards, def_by_full_name};
use serde_json::Value;

fn u8s(v: &Value) -> Vec<u8> {
    v.as_array().map(|a| a.iter().map(|x| x.as_u64().unwrap() as u8).collect()).unwrap_or_default()
}

/// (card name, attack names, Ability names) of an official card text (same reading as tools/official_names.py).
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
fn card_db_matches_twinleaf_dump() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/twinleaf-cards.json")).unwrap();
    let dump: Vec<Value> = serde_json::from_str(&text).unwrap();
    let pool: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/pool.json")).unwrap()).unwrap();
    let official: Value = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/official_text.json")).unwrap()).unwrap();
    let mut checked = 0;
    for row in &pool {
        let Some(name) = row["fullName"].as_str() else { continue };
        let c = dump.iter().find(|c| c["fullName"] == name).unwrap();
        let d = ptcg::carddb::def(def_by_full_name(name).expect("card in db"));
        assert_eq!(d.tl_name, c["name"].as_str().unwrap(), "{}", name);
        assert_eq!(d.tl_full_name, name);
        assert_eq!(d.tl_set, c["set"].as_str().unwrap(), "{}", name);
        assert_eq!(d.tl_set_number, c["setNumber"].as_str().unwrap(), "{}", name);
        // Official names: pool.json key / international set, official_text.json card, attack and Ability names.
        assert_eq!(d.full_name, row["key"].as_str().unwrap(), "{}", name);
        assert_eq!((d.set, d.set_number), (row["set"].as_str().unwrap(), row["number"].as_str().unwrap()), "{}", name);
        let off = official[&format!("{} {}", row["set"].as_str().unwrap(), row["number"].as_str().unwrap())]["text"].as_str().unwrap();
        let (oname, oatk, oabil) = parse_official(off);
        assert_eq!(d.name, oname, "{}", name);
        assert_eq!(d.attacks.iter().map(|a| a.name.to_string()).collect::<Vec<_>>(), oatk, "{} attack names", name);
        assert_eq!(d.powers.iter().map(|a| a.name.to_string()).collect::<Vec<_>>(), oabil, "{} Ability names", name);
        assert_eq!(d.super_type as u64, c["superType"].as_u64().unwrap(), "{}", name);
        assert_eq!(d.retreat, u8s(&c["retreat"]).as_slice(), "{} retreat", name);
        let tags: Vec<&str> = c["_tags"].as_array().unwrap().iter().map(|t| t.as_str().unwrap()).collect();
        assert_eq!(d.tag_names, tags.as_slice(), "{} tags", name);
        let atk = c["attacks"].as_array().unwrap();
        assert_eq!(d.attacks.len(), atk.len(), "{} attacks", name);
        for (a, j) in d.attacks.iter().zip(atk) {
            assert_eq!(a.tl_name, j["name"].as_str().unwrap());
            assert_eq!(a.cost, u8s(&j["cost"]).as_slice(), "{} cost", name);
            assert_eq!(a.damage as i64, j["damage"].as_i64().unwrap(), "{} damage", name);
        }
        let pw = c["powers"].as_array().unwrap();
        assert_eq!(d.powers.len(), pw.len(), "{} powers", name);
        for (p, j) in d.powers.iter().zip(pw) {
            assert_eq!(p.tl_name, j["name"].as_str().unwrap());
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
        checked += 1;
    }
    assert!(checked >= 650, "checked {}", checked);
    // The DB holds every pool printing plus each remapped row's previous
    // (name-based) printing, `prev_fullName` (tools/gen_carddb.py).
    let mut names: std::collections::BTreeSet<&str> = Default::default();
    for row in &pool {
        for k in ["fullName", "prev_fullName"] {
            if let Some(n) = row[k].as_str() {
                names.insert(n);
                assert!(def_by_full_name(n).is_some(), "{} in db", n);
            }
        }
    }
    // ...and the non-pool pre-evolutions in data/support_cards.json.
    let support: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../data/support_cards.json")).unwrap()).unwrap();
    for s in &support {
        let n = s["fullName"].as_str().unwrap();
        assert!(def_by_full_name(n).is_some(), "{} in db", n);
        names.insert(n);
    }
    assert_eq!(cards().len(), names.len());
}
