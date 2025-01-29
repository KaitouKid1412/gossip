#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico"); // Create an icon file and put it in assets folder
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {}
