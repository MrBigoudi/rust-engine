use engine::{core::application::ApplicationParameters, entry::engine_start, error};
use src::game::TestBedGame;

pub mod src;

fn main() {
    let application_parameters =
        ApplicationParameters::default().application_name(String::from("EngineTestBed"));
    let game = Box::new(TestBedGame);

    match engine_start(application_parameters, game) {
        Ok(()) => (),
        Err(err) => {
            error!("A runtime error occured: {:?}", err);
            panic!()
        }
    }
}
