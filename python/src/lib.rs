//! Python bindings: `Env` (single game) and `VecEnv` (N games
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

/// Random valid answer. Multi-pick decisions are built one pick at a time
/// over options that keep a valid answer reachable (`Game::pick_mask`), so
/// no blind retries are needed.
fn random_answer(g: &mut Game, sel: &SelectData, rng: &mut Rng) -> bool {
    let n = sel.options.len();
    if sel.max_count <= 1 {
        if n == 0 || sel.min_count == 0 && rng.index(n + 1) == n {
            if g.answer(sel, &[]).is_ok() {
                return true;
            }
        }
        return n > 0 && g.answer(sel, &[rng.index(n)]).is_ok();
    }
    let mut picks: Vec<usize> = Vec::new();
    loop {
        let (mask, stop) = g.pick_mask(sel, &picks);
        let allowed: Vec<usize> = (0..n).filter(|j| mask[*j]).collect();
        let k = allowed.len() + stop as usize;
        if k == 0 {
            break;
        }
        let r = rng.index(k);
        if r == allowed.len() {
            break;
        }
        picks.push(allowed[r]);
    }
    if g.answer(sel, &picks).is_ok() {
        return true;
    }
    // Search bound hit or a continuation failed: fall back to blind retries.
    for _ in 0..40 {
        let lo = sel.min_count.min(n);
        let hi = sel.max_count.min(n);
        let kk = if hi > lo { lo + rng.index(hi - lo + 1) } else { lo };
        let mut v: Vec<usize> = (0..n).collect();
        for i in (1..v.len()).rev() {
            v.swap(i, rng.index(i + 1));
        }
        v.truncate(kk);
        v.sort();
        if g.answer(sel, &v).is_ok() {
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
/// The learner plays seat 0 at every decision. A multi-pick decision
/// (`maxCount > 1`) is taken one pick per step: already-picked options drop
/// out (unless the prompt allows repeats, e.g. damage counters), and a STOP
/// option (type END) is appended once `minCount` picks are made. The answer is
/// submitted on STOP or at `maxCount` picks; if the engine rejects the
/// combination, a random valid answer is played instead and counted in
/// `invalid_answers`. Opponent decisions are uniformly random. Finished games
/// reset automatically.
#[pyclass]
struct VecEnv {
    envs: Vec<Env>,
    picks: Vec<Vec<usize>>,
    /// Cached learner view per env (None = recompute).
    views: std::sync::Mutex<Vec<Option<Vec<Option<usize>>>>>,
    rng: Rng,
    next_seed: u32,
    invalid: u64,
    /// Games ended because a prompt had no valid answer.
    stuck: u64,
    /// Games ended because the engine panicked (approved-divergence caps).
    aborted: u64,
    invalid_ctx: std::collections::BTreeMap<u8, u64>,
}

/// Option list the learner sees: `Some(i)` = engine option i, `None` = STOP.
/// Only picks that keep a valid answer reachable are offered; STOP only when
/// the picks so far are a valid answer.
fn view(game: &Game, sel: &SelectData, picks: &[usize]) -> Vec<Option<usize>> {
    if sel.select_type as u8 == 0 {
        // Turn actions are already legal by construction.
        return (0..sel.options.len()).map(Some).collect();
    }
    let (mask, stop) = game.pick_mask(sel, picks);
    let mut v: Vec<Option<usize>> = (0..sel.options.len()).filter(|j| mask[*j]).map(Some).collect();
    if stop {
        v.push(None);
    }
    v
}

impl VecEnv {
    fn view_of(&self, i: usize) -> Vec<Option<usize>> {
        let cached = self.views.lock().unwrap()[i].clone();
        if let Some(v) = cached {
            return v;
        }
        let v = match &self.envs[i].sel {
            Some(sel) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| view(&self.envs[i].game, sel, &self.picks[i]))).unwrap_or_default(),
            None => Vec::new(),
        };
        self.views.lock().unwrap()[i] = Some(v.clone());
        v
    }

    fn step_one(&mut self, i: usize, a: usize) -> PyResult<(f32, bool)> {
        if let Some(sel) = self.envs[i].sel.clone() {
            let v = self.view_of(i);
            self.views.lock().unwrap()[i] = None;
            let submitted = if v.is_empty() {
                // No pick keeps a valid answer reachable.
                Some(self.submit(i)?)
            } else {
                match v[a.min(v.len() - 1)] {
                    None => Some(self.submit(i)?),
                    Some(j) => {
                        self.picks[i].push(j);
                        // Submit when nothing more can be picked.
                        let done = sel.max_count <= 1 || {
                            let nv = self.view_of(i);
                            !nv.iter().any(|o| o.is_some())
                        };
                        if done {
                            Some(self.submit(i)?)
                        } else {
                            None
                        }
                    }
                }
            };
            if submitted == Some(false) {
                self.reset_env(i)?;
                return Ok((0.0, true));
            }
        }
        if self.picks[i].is_empty() {
            self.advance(i)
        } else {
            Ok((0.0, false))
        }
    }

    fn reset_env(&mut self, i: usize) -> PyResult<()> {
        let seed = self.next_seed;
        self.next_seed = self.next_seed.wrapping_add(1);
        self.picks[i].clear();
        self.views.lock().unwrap()[i] = None;
        self.envs[i].reset(seed)
    }

    /// Submit env `i`'s picks. `Ok(false)`: no valid answer exists (the game
    /// is stuck) and the caller must end it.
    fn submit(&mut self, i: usize) -> PyResult<bool> {
        self.views.lock().unwrap()[i] = None;
        let e = &mut self.envs[i];
        let sel = e.sel.clone().unwrap();
        let picks = std::mem::take(&mut self.picks[i]);
        if let Err(err) = e.game.answer(&sel, &picks) {
            if std::env::var_os("PTCG_DEBUG_REJECT").is_some() {
                let msg = sel.prompt_message(&e.game);
                eprintln!("reject {:?} ctx={:?} msg={} min={} max={} n={} picks={:?}: {:?}", sel.select_type, sel.context, msg, sel.min_count, sel.max_count, sel.options.len(), picks, err);
            }
            if !random_answer(&mut e.game, &sel, &mut self.rng) {
                if std::env::var_os("PTCG_DEBUG_REJECT").is_some() {
                    for j in 0..sel.options.len().min(4) {
                        let mut t = e.game.fork();
                        eprintln!("  stuck option {}: {:?}", j, t.answer(&sel, &[j]).err());
                    }
                }
                self.stuck += 1;
                return Ok(false);
            }
            self.invalid += 1;
            let key = sel.context as u8 + if picks.is_empty() { 100 } else { 0 };
            *self.invalid_ctx.entry(key).or_default() += 1;
        }
        e.refresh()?;
        Ok(true)
    }

    /// Advance env `i` until the learner must act or the game ends.
    fn advance(&mut self, i: usize) -> PyResult<(f32, bool)> {
        self.views.lock().unwrap()[i] = None;
        loop {
            let sel = match self.envs[i].sel.clone() {
                None => {
                    let w = self.envs[i].game.st.winner;
                    let r = if w == WINNER_P1 { 1.0 } else if w == WINNER_P2 { -1.0 } else { 0.0 };
                    self.reset_env(i)?;
                    return Ok((r, true));
                }
                Some(s) => s,
            };
            if sel.player == 0 && !sel.options.is_empty() && sel.max_count > 0 {
                return Ok((0.0, false));
            }
            if sel.player == 0 {
                // Nothing to choose (e.g. no legal cards): answer empty.
                if !self.submit(i)? {
                    self.reset_env(i)?;
                    return Ok((0.0, true));
                }
                continue;
            }
            let e = &mut self.envs[i];
            if !random_answer(&mut e.game, &sel, &mut self.rng) {
                self.reset_env(i)?;
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
        let mut v = VecEnv { envs, picks: vec![Vec::new(); n], views: std::sync::Mutex::new(vec![None; n]), rng: Rng::new(seed ^ 0x9e37), next_seed: seed.wrapping_add(n as u32), invalid: 0, stuck: 0, aborted: 0, invalid_ctx: Default::default() };
        for i in 0..n {
            v.advance(i)?;
        }
        Ok(v)
    }

    #[getter]
    fn num_envs(&self) -> usize {
        self.envs.len()
    }

    /// Multi-pick answers the engine rejected (replaced by a random valid one).
    #[getter]
    fn invalid_answers(&self) -> u64 {
        self.invalid
    }

    /// Games ended early because a prompt had no valid answer (stuck).
    #[getter]
    fn stuck_games(&self) -> u64 {
        self.stuck
    }

    /// Games ended because the engine panicked (a capacity cap hit by a runaway card bug).
    #[getter]
    fn aborted_games(&self) -> u64 {
        self.aborted
    }

    #[classattr]
    fn obs_size() -> usize {
        OBS_SIZE
    }

    #[classattr]
    fn option_features() -> usize {
        OPTION_FEATURES
    }

    /// Rejected multi-pick answers by select context id.
    fn invalid_by_context(&self) -> Vec<(u8, u64)> {
        self.invalid_ctx.iter().map(|(k, v)| (*k, *v)).collect()
    }

    /// (obs float32 [n, OBS_SIZE], options int32 [n, k_max, F], counts int32 [n]).
    /// Options are the learner's current view (see the class docs); a STOP
    /// option has type END (14).
    fn observe<'py>(&self, py: Python<'py>, k_max: usize) -> (Bound<'py, PyBytes>, Bound<'py, PyBytes>, Bound<'py, PyBytes>) {
        let mut obs = Vec::with_capacity(self.envs.len() * OBS_SIZE);
        let mut opts = vec![0i32; self.envs.len() * k_max * OPTION_FEATURES];
        let mut counts = Vec::with_capacity(self.envs.len());
        for (i, e) in self.envs.iter().enumerate() {
            obs.extend(observe(&e.game, 0));
            let mut c = 0;
            if let Some(sel) = &e.sel {
                for (k, o) in self.view_of(i).into_iter().take(k_max).enumerate() {
                    let f = match o {
                        Some(j) => option_features(sel, &sel.options[j]),
                        None => {
                            let stop = ptcg::interface::Opt { kind: 14, ..Default::default() };
                            option_features(sel, &stop)
                        }
                    };
                    let base = (i * k_max + k) * OPTION_FEATURES;
                    opts[base..base + OPTION_FEATURES].copy_from_slice(&f);
                    c += 1;
                }
            }
            counts.push(c as i32);
        }
        (f32_bytes(py, &obs), i32_bytes(py, &opts), i32_bytes(py, &counts))
    }

    /// Apply one option index (into the current view) per env; returns
    /// (rewards float32 [n], dones uint8 [n]).
    fn step<'py>(&mut self, py: Python<'py>, actions: Vec<usize>) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyBytes>)> {
        let mut rewards = Vec::with_capacity(self.envs.len());
        let mut dones = Vec::with_capacity(self.envs.len());
        for (i, a) in actions.into_iter().enumerate() {
            // An engine panic (a capacity cap hit by a runaway card bug) ends this
            // env's game instead of training.
            let (r, d) = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.step_one(i, a))) {
                Ok(res) => res?,
                Err(_) => {
                    self.aborted += 1;
                    self.picks[i].clear();
                    self.reset_env(i)?;
                    (0.0, true)
                }
            };
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
