use std::time;

pub struct ProfilingResult {
    durations: Vec<(String, time::Duration)>,
}

impl std::fmt::Display for ProfilingResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, duration) in &self.durations {
            writeln!(f, "{}: {:?}", name, duration)?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Profiler {
    start_times: Vec<(String, time::Instant)>,
}

impl Profiler {
    pub fn start(&mut self, name: &str) {
        self.start_times
            .push((name.to_string(), time::Instant::now()));
    }

    pub fn end(self) -> ProfilingResult {
        let mut durations = Vec::new();

        for (idx, start_time) in self.start_times.iter().enumerate() {
            let end_time = if idx < self.start_times.len() - 1 {
                self.start_times[idx + 1].1
            } else {
                time::Instant::now()
            };

            durations.push((start_time.0.clone(), end_time - start_time.1));
        }

        ProfilingResult { durations }
    }
}
