#[derive(Debug, Default)]
pub struct DemexUiStats {
    ui_dt: f64,
    ui_max_dt: f64,

    fixed_update_dt: f64,
    fixed_update_max_dt: f64,
}

impl DemexUiStats {
    pub fn ui_dt(&self) -> f64 {
        self.ui_dt
    }

    pub fn ui_max_dt(&self) -> f64 {
        self.ui_max_dt
    }

    pub fn fixed_update_dt(&self) -> f64 {
        self.fixed_update_dt
    }

    pub fn fixed_update_max_dt(&self) -> f64 {
        self.fixed_update_max_dt
    }
}

impl DemexUiStats {
    pub fn ui(&mut self, dt: f64) {
        self.ui_dt = dt;
        self.ui_max_dt = self.ui_max_dt.max(dt);
    }

    pub fn fixed_update(&mut self, dt: f64) {
        self.fixed_update_dt = dt;
        self.fixed_update_max_dt = self.fixed_update_max_dt.max(dt);
    }
}
