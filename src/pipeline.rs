use std::{sync::Mutex, thread, sync::Arc};

use crossbeam::channel::{unbounded, Receiver, Sender};


trait Node {
    fn step(&mut self);
}

pub struct S<I, O> {
    pub i: Option<Receiver<I>>,
    pub o: Sender<O>,
    pub f: Box<dyn Fn(I) -> O + Send>,
}

pub struct PipeLineInternal<I, O> {
    pub i: Option<Receiver<I>>,
    pub o: Sender<O>,
    pub r: Receiver<O>,
}

pub struct PipeLine<I, O> {
    pub inner: Arc<Mutex<PipeLineInternal<I, O>>>,
}

impl<I, O> PipeLine<I, O>
where
    I: Send + 'static,
    O: Send + 'static,
{
    pub fn new(f: impl Fn(Sender<O>, Option<Receiver<I>>) + Send + 'static) -> Self {
        let (s, r) = unbounded();

        let inner = Arc::new(Mutex::new(PipeLineInternal {
            i: None,
            o: s,
            r,
        }));

        let _inner_clone = Arc::clone(&inner);
        let oc = _inner_clone.lock().unwrap().o.clone();
        let ic = _inner_clone.lock().unwrap().i.clone();
        
        thread::spawn(move || {
            f(oc, ic);
        });

        PipeLine { inner }
    }
}

pub fn join<I0, O0, O1>(a: PipeLine<I0, O0>, b: PipeLine<O0, O1>) -> PipeLine<I0, O1> {
    b.inner.lock().unwrap().i = Some(a.inner.lock().unwrap().r.clone());
    let inner = Arc::new(Mutex::new(PipeLineInternal {
        i: a.inner.lock().unwrap().i.clone(),
        o: b.inner.lock().unwrap().o.clone(),
        r: b.inner.lock().unwrap().r.clone(),
    }));

    PipeLine { inner }
}