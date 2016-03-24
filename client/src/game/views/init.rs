// Internal Dependencies ------------------------------------------------------
use game::{Game, ClientHandle};
use self::super::{View, MenuView};


// View Implementation --------------------------------------------------------
#[derive(Debug)]
pub struct InitView;

impl View for InitView {

    fn name(&self) -> &str {
        "Init"
    }

    fn init(&mut self, game: &mut Game, _: &mut ClientHandle) {
        game.set_view(Box::new(MenuView))
    }

}

