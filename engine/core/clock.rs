use crate::platforms::platform::Platform;

use super::errors::EngineError;

#[derive(Default)]
pub(crate) struct Clock {
    pub start_time: f64,
    pub elapsed_time: f64,
}

impl Clock {
    // Updates the provided clock. Should be called just before checking elapsed time.
    // Has no effect on non-started clocks.
    pub fn update(&mut self, platform: &dyn Platform) -> Result<(), EngineError> {
        if self.start_time != 0. {
            self.elapsed_time = platform.get_absolute_time_in_seconds()? - self.start_time;
        }
        Ok(())
    }

    // Starts the provided clock. Resets elapsed time.
    pub fn start(&mut self, platform: &dyn Platform) -> Result<(), EngineError> {
        self.start_time = platform.get_absolute_time_in_seconds()?;
        self.elapsed_time = 0.;
        Ok(())
    }

    // Stops the provided clock. Does not reset elapsed time.
    pub fn stop(&mut self) {
        self.start_time = 0.;
    }
}
