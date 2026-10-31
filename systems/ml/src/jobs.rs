//! Jobs: a run on a worker, named by what it computes, joined later.

use super::*;

pub(crate) type Outputs = Vec<(String, Tensor)>;

pub(crate) enum JobState {
    Running,
    Done(Result<Arc<Outputs>, String>),
}

pub(crate) struct Job {
    pub(crate) state: Mutex<JobState>,
    pub(crate) ready: Condvar,
    /// when it was last asked for (finished jobs are let go a while after)
    pub(crate) touched: Mutex<Instant>,
}

#[derive(Default)]
pub(crate) struct Jobs {
    pub(crate) map: HashMap<String, Arc<Job>>,
    pub(crate) queue: std::collections::VecDeque<(String, Arc<Model>, Vec<(String, Tensor)>)>,
}

pub(crate) struct Pool {
    pub(crate) jobs: Mutex<Jobs>,
    pub(crate) work: Condvar,
}

pub(crate) fn pool() -> &'static Arc<Pool> {
    static P: OnceLock<Arc<Pool>> = OnceLock::new();
    P.get_or_init(|| {
        let p = Arc::new(Pool { jobs: Mutex::new(Jobs::default()), work: Condvar::new() });
        // two workers: a heavy job doesn't hold up every other model's
        for i in 0..2 {
            let p = p.clone();
            std::thread::Builder::new().name(format!("raea-ml-{i}")).spawn(move || worker(&p)).expect("ml worker");
        }
        p
    })
}

pub(crate) fn worker(p: &Pool) {
    loop {
        let (key, model, feeds) = {
            let mut j = p.jobs.lock().unwrap();
            loop {
                if let Some(x) = j.queue.pop_front() {
                    break x;
                }
                j = p.work.wait(j).unwrap();
            }
        };
        let r = model.run(&feeds, None).map(Arc::new).map_err(|e| format!("{e:#}"));
        let job = p.jobs.lock().unwrap().map.get(&key).cloned();
        if let Some(job) = job {
            *job.state.lock().unwrap() = JobState::Done(r);
            job.ready.notify_all();
        }
    }
}

/// The name of a run: the model's file and options and every input's name, type, shape and bytes (FNV-1a, 128 bits).
pub fn job_key(model: &Model, feeds: &[(String, Tensor)]) -> String {
    let mut a: u64 = 0xcbf29ce484222325;
    let mut b: u64 = 0x84222325cbf29ce4;
    let mut eat = |bytes: &[u8]| {
        for &x in bytes {
            a = (a ^ x as u64).wrapping_mul(0x100000001b3);
            b = (b ^ x as u64).wrapping_mul(0x100000001b3).rotate_left(5);
        }
    };
    eat(model.file.to_string_lossy().as_bytes());
    eat(format!("{:?}", model.options).as_bytes());
    let mut sorted: Vec<&(String, Tensor)> = feeds.iter().collect();
    sorted.sort_by(|x, y| x.0.cmp(&y.0));
    for (name, t) in sorted {
        eat(name.as_bytes());
        eat(t.dtype.name().as_bytes());
        for d in &t.shape {
            eat(&d.to_le_bytes());
        }
        eat(&t.bytes);
    }
    format!("{a:016x}{b:016x}")
}

/// Starts a run on a worker (or finds the one already started for the same model and inputs) and returns its name.
pub fn request(model: Arc<Model>, feeds: Vec<(String, Tensor)>) -> String {
    let key = job_key(&model, &feeds);
    let p = pool();
    let mut j = p.jobs.lock().unwrap();
    // finished jobs nobody asked for in a minute are let go
    j.map.retain(|_, job| !matches!(*job.state.lock().unwrap(), JobState::Done(_)) || job.touched.lock().unwrap().elapsed() < Duration::from_secs(60));
    if let Some(job) = j.map.get(&key) {
        *job.touched.lock().unwrap() = Instant::now();
        return key;
    }
    j.map.insert(key.clone(), Arc::new(Job { state: Mutex::new(JobState::Running), ready: Condvar::new(), touched: Mutex::new(Instant::now()) }));
    j.queue.push_back((key.clone(), model, feeds));
    drop(j);
    p.work.notify_one();
    key
}

/// A job's outputs, waiting for it to finish (a pure function of what was requested: safe to fold into a game).
pub fn result(key: &str) -> anyhow::Result<Arc<Outputs>> {
    let job = pool().jobs.lock().unwrap().map.get(key).cloned().ok_or_else(|| anyhow!("no job {key} (request it first; finished jobs are kept a minute after their last use)"))?;
    *job.touched.lock().unwrap() = Instant::now();
    let mut s = job.state.lock().unwrap();
    loop {
        match &*s {
            JobState::Done(r) => return r.clone().map_err(|e| anyhow!(e)),
            JobState::Running => s = job.ready.wait(s).unwrap(),
        }
    }
}

/// A job's outputs if it has finished, without waiting. Display-only: whether it's done by a given frame depends on
/// the machine and what else it's doing, so a game must not fold this into its state.
pub fn poll(key: &str) -> Option<anyhow::Result<Arc<Outputs>>> {
    let job = pool().jobs.lock().unwrap().map.get(key).cloned()?;
    let s = job.state.lock().unwrap();
    match &*s {
        JobState::Done(r) => Some(r.clone().map_err(|e| anyhow!(e))),
        JobState::Running => None,
    }
}
