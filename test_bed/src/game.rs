use engine::game::Game;

pub struct TestBedGame;

impl Game for TestBedGame {
    fn initialize(&mut self) -> Result<(), engine::core::errors::EngineError> {
        // TODO:
        Ok(())
    }

    fn update(&mut self, _delta_time: f32) -> Result<(), engine::core::errors::EngineError> {
        // TODO:
        Ok(())
    }

    fn render(&self, _delta_time: f32) -> Result<(), engine::core::errors::EngineError> {
        // TODO:
        Ok(())
    }

    fn resize(
        &mut self,
        _new_width: u16,
        _new_height: u16,
    ) -> Result<(), engine::core::errors::EngineError> {
        // TODO:
        todo!()
    }

    fn shutdown(&mut self) -> Result<(), engine::core::errors::EngineError> {
        // TODO:
        Ok(())
    }
}
