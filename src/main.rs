mod messagepack_helpers;
mod request_handler;
mod requests;
mod responses;
mod websocket_server;

use std::error::Error;

use muda::{Menu, PredefinedMenuItem, Submenu};
use wry::application::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::webview::WebViewBuilder;

const TITLE: &str = "wry_websocket_rpc_server";

fn init_menu() -> Menu {
    let menu = Menu::new();
    let app_m = Submenu::new("App", true);
    menu.append(&app_m).unwrap();
    app_m
        .append_items(&[
            &PredefinedMenuItem::about(None, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ])
        .unwrap();
    let edit_m = Submenu::new("&Edit", true);
    menu.append_items(&[&edit_m]).unwrap();
    let copy_i = PredefinedMenuItem::copy(None);
    let cut_i = PredefinedMenuItem::cut(None);
    let paste_i = PredefinedMenuItem::paste(None);
    let select_all_i = PredefinedMenuItem::select_all(None);
    edit_m
        .append_items(&[&copy_i, &cut_i, &paste_i, &select_all_i])
        .unwrap();
    menu.init_for_nsapp();
    menu
}

fn main() -> Result<(), Box<dyn Error>> {
    // logger
    env_logger::init();
    // websocket server
    async_std::task::spawn(async {
        websocket_server::start("127.0.0.1:3000").await;
    });
    // menu
    let _menu = init_menu();
    // event loop
    let event_loop = EventLoopBuilder::new().build();
    // window
    let window = WindowBuilder::new()
        .with_title(TITLE)
        .with_inner_size(PhysicalSize {
            width: 1280,
            height: 1024,
        })
        .build(&event_loop)
        .unwrap();
    // webview
    let _webview = WebViewBuilder::new(window)?
        .with_html(r#"
            <!doctype html>
            <html>
                <head></head>
                <body>
                    <p>Hello, world!</p>
                </body>
            </html>
        "#)?
        .with_devtools(true)
        .build()?;
    // run
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}
