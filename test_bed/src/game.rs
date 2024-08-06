use engine::game::Game;

pub struct TestBedGame;

impl Game for TestBedGame {
    fn initialize(&mut self) -> Result<(), engine::core::debug::errors::EngineError> {
        Ok(())
    }

    fn update(&mut self, _delta_time: f64) -> Result<(), engine::core::debug::errors::EngineError> {
        Ok(())
    }

    fn render(&self, _delta_time: f64) -> Result<(), engine::core::debug::errors::EngineError> {
        Ok(())
    }

    fn resize(
        &mut self,
        _new_width: u32,
        _new_height: u32,
    ) -> Result<(), engine::core::debug::errors::EngineError> {
        // TODO: implement window resizing
        // todo!()
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), engine::core::debug::errors::EngineError> {
        Ok(())
    }
}
