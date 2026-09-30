//! Python bindings (PLAN.md 3.6): `Env` (single game) and `VecEnv` (N games
//! stepped in one call). Arrays cross the boundary as little-endian bytes and
//! are wrapped with `numpy.frombuffer` by the pure-Python layer in `ptcg/`.

use ptcg::carddb::{def_by_full_name, DefId};
use ptcg::game::Game;
use ptcg::interface::SelectData;
use ptcg::obs::{observe, option_features, OBS_SIZE, OPTION_FEATURES};
use ptcg::rng::Rng;
use ptcg::types::*;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

fn parse_deck(names: &[String]) -> PyResult<Vec<DefId>> {
    let mut out = Vec::with_capacity(names.len());
    for n in names {
        out.push(def_by_full_name(n).ok_or_else(|| PyValueError::new_err(format!("unknown card: {}", n)))?);
    }
    Ok(out)
}

fn err(e: ptcg::GameError) -> PyErr {
    PyValueError::new_err(e.0)
}

fn f32_bytes<'py>(py: Python<'py>, v: &[f32]) -> Bound<'py, PyBytes> {
    let mut b = Vec::with_capacity(v.len() * 4);
    for x in v {
        b.extend_from_slice(&x.to_le_bytes());
    }
    PyBytes::new(py, &b)
}

fn i32_bytes<'py>(py: Python<'py>, v: &[i32]) -> Bound<'py, PyBytes> {
    let mut b = Vec::with_capacity(v.len() * 4);
    for x in v {
        b.extend_from_slice(&x.to_le_bytes());
    }
    PyBytes::new(py, &b)
}

fn select_dict<'py>(py: Python<'py>, sel: &SelectData) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    d.set_item("player", sel.player)?;
    d.set_item("type", sel.select_type as u8)?;
    d.set_item("context", sel.context as u8)?;
    d.set_item("minCount", sel.min_count)?;
    d.set_item("maxCount", sel.max_count)?;
    let opts = PyList::empty(py);
    for o in &sel.options {
        let od = PyDict::new(py);
        od.set_item("type", o.kind)?;
        od.set_item("number", o.number)?;
        od.set_item("area", o.area)?;
        od.set_item("index", o.index)?;
        od.set_item("playerIndex", o.player_index)?;
        od.set_item("inPlayArea", o.in_play_area)?;
        od.set_item("inPlayIndex", o.in_play_index)?;
        od.set_item("attackId", o.attack_id)?;
        od.set_item("cardId", o.card_id)?;
        od.set_item("serial", o.serial)?;
        opts.append(od)?;
    }
    d.set_item("option", opts)?;
    Ok(d)
}

/// Uniform random valid answer (retries until the engine accepts one).
fn random_answer(g: &mut Game, sel: &SelectData, rng: &mut Rng) -> bool {
    let n = sel.options.len();
    for _ in 0..40 {
        let lo = sel.min_count.min(n);
        let hi = sel.max_count.min(n);
        let k = if hi > lo { lo + rng.index(hi - lo + 1) } else { lo };
        let mut idx: Vec<usize> = (0..n).collect();
        for i in (1..idx.len()).rev() {
            idx.swap(i, rng.index(i + 1));
        }
        idx.truncate(k);
        idx.sort();
        if g.answer(sel, &idx).is_ok() {
            return true;
        }
    }
    false
}

#[pyclass]
#[derive(Clone)]
struct Env {
    game: Box<Game>,
    decks: [Vec<DefId>; 2],
    manual_chance: bool,
    sel: Option<SelectData>,
}

impl Env {
    fn refresh(&mut self) -> PyResult<()> {
        self.sel = self.game.select_with(self.manual_chance).map_err(err)?;
        Ok(())
    }
}

#[pymethods]
impl Env {
    #[new]
    #[pyo3(signature = (deck_a, deck_b, seed=0, manual_chance=false))]
    fn new(deck_a: Vec<String>, deck_b: Vec<String>, seed: u32, manual_chance: bool) -> PyResult<Self> {
        let decks = [parse_deck(&deck_a)?, parse_deck(&deck_b)?];
        let mut e = Env { game: Box::new(Game::new(seed)), decks, manual_chance, sel: None };
        e.reset(seed)?;
        Ok(e)
    }

    /// Start a new game with `seed`.
    fn reset(&mut self, seed: u32) -> PyResult<()> {
        *self.game = Game::new(seed);
        self.game.start([&self.decks[0], &self.decks[1]]).map_err(err)?;
        self.refresh()
    }

    /// Current decision as a cabt-style dict, or None when the game is over.
    fn select<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyDict>>> {
        match &self.sel {
            Some(s) => Ok(Some(select_dict(py, s)?)),
            None => Ok(None),
        }
    }

    /// Answer the current decision with option indices.
    fn step(&mut self, indices: Vec<usize>) -> PyResult<()> {
        let sel = self.sel.clone().ok_or_else(|| PyValueError::new_err("game over"))?;
        self.game.answer(&sel, &indices).map_err(err)?;
        self.refresh()
    }

    /// Answer a manual shuffle with a permutation of deck indices.
    fn step_shuffle(&mut self, order: Vec<u8>) -> PyResult<()> {
        let sel = self.sel.clone().ok_or_else(|| PyValueError::new_err("game over"))?;
        self.game.answer_shuffle(&sel, &order).map_err(err)?;
        self.refresh()
    }

    /// Play a uniformly random valid answer (seeded).
    fn step_random(&mut self, seed: u32) -> PyResult<bool> {
        let sel = match self.sel.clone() {
            Some(s) => s,
            None => return Ok(false),
        };
        let mut rng = Rng::new(seed);
        let ok = random_answer(&mut self.game, &sel, &mut rng);
        self.refresh()?;
        Ok(ok)
    }

    /// Observation bytes (float32 x OBS_SIZE) for `player`.
    fn observe_bytes<'py>(&self, py: Python<'py>, player: usize) -> Bound<'py, PyBytes> {
        f32_bytes(py, &observe(&self.game, player))
    }

    /// Option features bytes (int32 x K x OPTION_FEATURES).
    fn option_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
        let mut v = Vec::new();
        if let Some(sel) = &self.sel {
            for o in &sel.options {
                v.extend_from_slice(&option_features(sel, o));
            }
        }
        i32_bytes(py, &v)
    }

    #[getter]
    fn done(&self) -> bool {
        self.sel.is_none()
    }
    #[getter]
    fn winner(&self) -> i8 {
        self.game.st.winner
    }
    #[getter]
    fn turn(&self) -> i32 {
        self.game.st.turn
    }
    #[getter]
    fn to_move(&self) -> Option<u8> {
        self.sel.as_ref().map(|s| s.player)
    }

    /// Independent copy (a memcpy of the whole game, mid-prompt included).
    fn clone(&self) -> Env {
        Clone::clone(self)
    }

    /// Re-deal hidden cards uniformly for `viewer` and reseed chance.
    fn determinize(&mut self, viewer: usize, seed: u32) -> PyResult<()> {
        ptcg::search::determinize(&mut self.game, viewer, seed).map_err(err)?;
        self.refresh()
    }

    fn state_hash(&self) -> String {
        self.game.state_hash()
    }
    fn canonical(&self) -> String {
        self.game.canonical_text()
    }
}

/// N independent games against a random opponent, stepped together.
///
/// The learner plays seat 0 at every decision of type MAIN or with a single
/// pick; other decisions (multi-select prompts) and all opponent decisions are
/// answered uniformly at random. Finished games reset automatically.
#[pyclass]
struct VecEnv {
    envs: Vec<Env>,
    rng: Rng,
    next_seed: u32,
}

impl VecEnv {
    /// Advance env `i` until the learner must act or the game ends.
    fn advance(&mut self, i: usize) -> PyResult<(f32, bool)> {
        loop {
            let e = &mut self.envs[i];
            let sel = match e.sel.clone() {
                None => {
                    let w = e.game.st.winner;
                    let r = if w == WINNER_P1 { 1.0 } else if w == WINNER_P2 { -1.0 } else { 0.0 };
                    let seed = self.next_seed;
                    self.next_seed = self.next_seed.wrapping_add(1);
                    e.reset(seed)?;
                    return Ok((r, true));
                }
                Some(s) => s,
            };
            let learner = sel.player == 0 && (sel.max_count <= 1 || sel.select_type as u8 == 0);
            if learner && !sel.options.is_empty() {
                return Ok((0.0, false));
            }
            if !random_answer(&mut e.game, &sel, &mut self.rng) {
                // No valid answer found: count as a draw and restart.
                let seed = self.next_seed;
                self.next_seed = self.next_seed.wrapping_add(1);
                e.reset(seed)?;
                return Ok((0.0, true));
            }
            e.refresh()?;
        }
    }
}

#[pymethods]
impl VecEnv {
    #[new]
    #[pyo3(signature = (n, deck_a, deck_b, seed=0))]
    fn new(n: usize, deck_a: Vec<String>, deck_b: Vec<String>, seed: u32) -> PyResult<Self> {
        let mut envs = Vec::with_capacity(n);
        for i in 0..n {
            envs.push(Env::new(deck_a.clone(), deck_b.clone(), seed.wrapping_add(i as u32), false)?);
        }
        let mut v = VecEnv { envs, rng: Rng::new(seed ^ 0x9e37), next_seed: seed.wrapping_add(n as u32) };
        for i in 0..n {
            v.advance(i)?;
        }
        Ok(v)
    }

    #[getter]
    fn num_envs(&self) -> usize {
        self.envs.len()
    }

    #[classattr]
    fn obs_size() -> usize {
        OBS_SIZE
    }

    #[classattr]
    fn option_features() -> usize {
        OPTION_FEATURES
    }

    /// (obs float32 [n, OBS_SIZE], options int32 [n, k_max, F], counts int32 [n]).
    fn observe<'py>(&self, py: Python<'py>, k_max: usize) -> (Bound<'py, PyBytes>, Bound<'py, PyBytes>, Bound<'py, PyBytes>) {
        let mut obs = Vec::with_capacity(self.envs.len() * OBS_SIZE);
        let mut opts = vec![0i32; self.envs.len() * k_max * OPTION_FEATURES];
        let mut counts = Vec::with_capacity(self.envs.len());
        for (i, e) in self.envs.iter().enumerate() {
            obs.extend(observe(&e.game, 0));
            let mut c = 0;
            if let Some(sel) = &e.sel {
                for (k, o) in sel.options.iter().take(k_max).enumerate() {
                    let f = option_features(sel, o);
                    let base = (i * k_max + k) * OPTION_FEATURES;
                    opts[base..base + OPTION_FEATURES].copy_from_slice(&f);
                    c += 1;
                }
            }
            counts.push(c as i32);
        }
        (f32_bytes(py, &obs), i32_bytes(py, &opts), i32_bytes(py, &counts))
    }

    /// Apply one option index per env; returns (rewards float32 [n], dones uint8 [n]).
    fn step<'py>(&mut self, py: Python<'py>, actions: Vec<usize>) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> {
        let mut rewards = Vec::with_capacity(self.envs.len());
        let mut dones = Vec::with_capacity(self.envs.len());
        for (i, a) in actions.into_iter().enumerate() {
            {
                let e = &mut self.envs[i];
                if let Some(sel) = e.sel.clone() {
                    let k = a.min(sel.options.len().saturating_sub(1));
                    if e.game.answer(&sel, &[k]).is_err() {
                        random_answer(&mut e.game, &sel, &mut self.rng);
                    }
                    e.refresh()?;
                }
            }
            let (r, d) = self.advance(i)?;
            rewards.push(r);
            dones.push(d as u8);
        }
        Ok((f32_bytes(py, &rewards), PyBytes::new(py, &dones)))
    }
}

#[pymodule]
fn _ptcg(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Env>()?;
    m.add_class::<VecEnv>()?;
    m.add("OBS_SIZE", OBS_SIZE)?;
    m.add("OPTION_FEATURES", OPTION_FEATURES)?;
    Ok(())
}
