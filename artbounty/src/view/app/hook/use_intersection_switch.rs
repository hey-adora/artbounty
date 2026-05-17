use leptos::prelude::*;

use crate::{
    api::{Api, Server404Err, ServerErr},
    path::link_img,
};
use tracing::{error, info, trace, warn};

#[derive(Clone, Copy)]
pub struct IntersectionSwitch {
    pub is_enabled: StoredValue<bool, LocalStorage>,
}

impl IntersectionSwitch {
    pub fn new() -> Self {
        Self {
            is_enabled: StoredValue::new_local(false),
        }
    }

    pub fn is_enabled(&self, is_intersecting: bool) -> bool {
        if !is_intersecting {
            self.is_enabled.set_value(true);
            return false;
        }

        let is_enabled = self.is_enabled.get_value();

        if is_enabled {
            self.is_enabled.set_value(false);
        }

        is_enabled
    }

    pub fn reset(&self) {
        self.is_enabled.set_value(false);
    }

}

#[cfg(test)]
pub mod tests {
    use crate::init_test_log;
    use crate::view::app::hook::use_intersection_switch::IntersectionSwitch;
    use hydration_context::HydrateSharedContext;
    use leptos::prelude::*;
    use std::sync::Arc;

    #[tokio::test]
    pub async fn hook_intersection_switch() {
        init_test_log();
        let owner = Owner::new_root(Some(Arc::new(HydrateSharedContext::new())));

        let switch = IntersectionSwitch::new();
        let output = switch.is_enabled(false);
        assert!(!output);
        let output = switch.is_enabled(true);
        assert!(output);
        let output = switch.is_enabled(true);
        assert!(!output);
        let output = switch.is_enabled(true);
        assert!(!output);
        let output = switch.is_enabled(false);
        assert!(!output);
        let output = switch.is_enabled(false);
        assert!(!output);
        let output = switch.is_enabled(true);
        assert!(output);

        let switch = IntersectionSwitch::new();
        let output = switch.is_enabled(true);
        assert!(!output);
        let output = switch.is_enabled(true);
        assert!(!output);
        let output = switch.is_enabled(true);
        assert!(!output);
        let output = switch.is_enabled(false);
        assert!(!output);
        let output = switch.is_enabled(false);
        assert!(!output);
        let output = switch.is_enabled(true);
        assert!(output);
        let output = switch.is_enabled(true);
        assert!(!output);
    }
}
