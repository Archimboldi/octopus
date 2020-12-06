#![windows_subsystem = "windows"]
extern crate sciter;

fn main() {
    // let html = include_bytes!("../ui/clock.htm");
    let resouces = include_bytes!("../ui.rc");
    let mut frame = sciter::WindowBuilder::main_window().with_size((400, 300)).create();
    // let load = format!("file://{}/ui/clock.htm", std::env::current_dir().unwrap().to_str().unwrap());
    // frame.load_html(html, Some(&load));
    frame.archive_handler(resouces).expect("Invalid archive");
    frame.load_file("this://app/clock.htm");
    frame.run_app();
}