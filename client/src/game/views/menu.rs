// Internal Dependencies ------------------------------------------------------
use game::{Game, ClientHandle};
use shared::{Color, ColorName};
use renderer::KeyCode;
use self::super::{View, ConnectView};


// View Implementation --------------------------------------------------------
#[derive(Debug)]
pub struct MenuView;

impl View for MenuView {

    fn name(&self) -> &str {
        "Menu"
    }

    fn push(&mut self, game: &mut Game, handle: &mut ClientHandle) {
        game.reset(handle);
    }

    fn draw(&mut self, game: &mut Game, handle: &mut ClientHandle) {

        handle.renderer.clear(&Color::from_name(ColorName::Black));

        handle.renderer.text(
            &Color::from_name(ColorName::White),
            0.0, 0.0,
            &format!("Menu - Press Enter to connect")[..]
        );

        if handle.renderer.key_released(KeyCode::Enter) {
            let view = Box::new(ConnectView::new(game.server_addr));
            game.set_view(view);
        }

    }

}

