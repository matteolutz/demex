use std::borrow::BorrowMut;

use gpui::{App, BorrowAppContext, Global};

use crate::ui2::{panels::DockWindowPanelType, utils::profiler::DemexUiProfiler};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DemexUiProfilerGlobalKey {
    Panel(DockWindowPanelType),
}

impl From<DockWindowPanelType> for DemexUiProfilerGlobalKey {
    fn from(value: DockWindowPanelType) -> Self {
        Self::Panel(value)
    }
}

impl Global for DemexUiProfiler<DemexUiProfilerGlobalKey> {}

pub trait DemexUiProfilerGlobalAppExt {
    fn profiler(&self) -> &DemexUiProfiler<DemexUiProfilerGlobalKey>;

    fn update_profiler<F>(&mut self, f: F)
    where
        F: FnOnce(&mut DemexUiProfiler<DemexUiProfilerGlobalKey>, &mut App);
}

impl DemexUiProfilerGlobalAppExt for App {
    fn profiler(&self) -> &DemexUiProfiler<DemexUiProfilerGlobalKey> {
        self.global()
    }

    fn update_profiler<F>(&mut self, f: F)
    where
        F: FnOnce(&mut DemexUiProfiler<DemexUiProfilerGlobalKey>, &mut App),
    {
        self.update_global(f)
    }
}

pub trait DemexUiProfilerGlobalBorrowAppExt {
    fn with_profiler<F, R>(&mut self, key: DemexUiProfilerGlobalKey, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

impl<C: BorrowMut<App>> DemexUiProfilerGlobalBorrowAppExt for C {
    #[inline]
    fn with_profiler<F, R>(&mut self, key: DemexUiProfilerGlobalKey, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let app = self.borrow_mut();
        app.update_profiler(|profiler, _| profiler.start(key.clone()));

        let result = f(self);

        let app = self.borrow_mut();
        app.update_profiler(|profiler, _| profiler.end(key));

        result
    }
}
