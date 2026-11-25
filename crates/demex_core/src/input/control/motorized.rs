use std::time;

#[derive(Debug, Clone)]
pub struct DemexInputMotorizedControlState {
    last_input: Option<time::Instant>,
    value_to_send: Option<f32>,

    debounce_time: f32,
}

impl Default for DemexInputMotorizedControlState {
    fn default() -> Self {
        Self::new(0.1)
    }
}

impl DemexInputMotorizedControlState {
    pub fn new(debounce_time_s: f32) -> Self {
        Self {
            debounce_time: debounce_time_s,
            last_input: None,
            value_to_send: None,
        }
    }
}

impl DemexInputMotorizedControlState {
    pub fn input(&mut self) {
        self.last_input = Some(time::Instant::now());
    }

    pub fn update_value(&mut self, value: f32) {
        self.value_to_send = Some(value)
    }

    fn consume_value(&mut self) -> Option<f32> {
        let val = self.value_to_send;
        self.value_to_send = None;
        val
    }

    pub fn send_value(&mut self) -> Option<f32> {
        if let Some(last_input) = self.last_input {
            if last_input.elapsed().as_secs_f32() > self.debounce_time {
                self.consume_value()
            } else {
                None
            }
        } else {
            self.consume_value()
        }
    }
}

pub trait DemexInputMotorizedControlStateListTrait {
    fn send_values(&mut self) -> Vec<(usize, f32)>;
}

impl DemexInputMotorizedControlStateListTrait for [DemexInputMotorizedControlState] {
    fn send_values(&mut self) -> Vec<(usize, f32)> {
        self.iter_mut()
            .enumerate()
            .filter_map(|(idx, state)| state.send_value().map(|value| (idx, value)))
            .collect::<Vec<_>>()
    }
}
