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


    #[track_caller]
    pub fn spawn<Fut, T>(self, callback: Fut)
        where
            Fut: Future<Output = T> + 'static
    {

        let is_busy = self.is_busy.clone();
        if is_busy.get_untracked() {
            warn!("trying to spawn while busy");
            return;
        }
        is_busy.set(true);
        spawn_local(async move {
            trace!("executing callback");
            let _ = callback.await;
            trace!("finished callback");
            let result = is_busy.try_set(false);
            if result.is_some() {
                warn!("spawner already disposed");
            }
        });


    }
}
