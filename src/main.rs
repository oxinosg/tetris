mod utils;

use log::info;
use stdweb::web::{document, IParentNode};
use tetris::{Model, Msg};
use yew::App;

fn main() {
    yew::initialize();
    utils::set_panic_hook();
    web_logger::init();
    let app = App::<tetris::Model>::new();
    let element = document().query_selector(".tetris-app").unwrap().unwrap();
    app.mount(element);
    info!("starting up");
    yew::run_loop();
}
