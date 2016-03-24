// Internal Dependencies ------------------------------------------------------
use game::{Game, ClientHandle};
use shared::{Color, ColorName};
use self::super::{View, ConnectView};


// View Implementation --------------------------------------------------------
#[derive(Debug)]
pub struct MenuView;

impl View for MenuView {

    fn name(&self) -> &str {
        "Menu"
    }

    fn push(&mut self, game: &mut Game, client: &mut ClientHandle) {
        game.reset(client);
    }

    fn draw(&mut self, game: &mut Game, client: &mut ClientHandle) {

        client.renderer.clear(&Color::from_name(ColorName::Black));

        client.renderer.text(
            &Color::from_name(ColorName::White),
            0.0, 0.0,
            &format!("Menu")[..]
        );

        if client.renderer.key_released(67) {
            let view = Box::new(ConnectView::new(game.server_addr));
            game.set_view(view);
        }

    }

}

