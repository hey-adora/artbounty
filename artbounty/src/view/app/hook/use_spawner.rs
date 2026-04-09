use leptos::{prelude::*, task::spawn_local};
use tracing::{warn, trace};

#[derive(Clone, Copy, Default)]
pub struct Spawner {
    pub is_busy: RwSignal<bool, LocalStorage>,
}

impl Spawner {
    pub fn new() -> Self {
        Self::default()
    }


    pub fn spawn<Fut, T>(&self, callback: Fut)
        where
            Fut: Future<Output = T> + 'static
    {

        let is_busy = self.is_busy.clone();
        if is_busy.get_untracked() {
            warn!("trying to spawn while busy");
        }
        is_busy.set(true);
        spawn_local(async move {
            trace!("executing callback");
            let _ = callback.await;
            trace!("finished callback");
            is_busy.set(false);
        });


    }
}
