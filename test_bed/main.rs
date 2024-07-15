use engine::{core::application::ApplicationParameters, entry::start_engine, error, game::Game};
use src::game::TestBedGame;

pub mod src;

fn create_game() -> Box<dyn Game> {
    Box::new(TestBedGame)
}

fn create_application_configuration() -> ApplicationParameters {
    ApplicationParameters::default().application_name(String::from("EngineTestBed"))
}

fn main() {
    let application_parameters = create_application_configuration();
    let game = create_game();

    match start_engine(application_parameters, game) {
        Ok(()) => (),
        Err(err) => {
            error!("A runtime error occured: {:?}", err);
            panic!()
        }
    }
}
