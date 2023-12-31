#![windows_subsystem = "windows"]
use m3u8_rs::Playlist;
use rodio::Decoder;
use std::{process::{Command, exit}, thread};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, MenuId},
    TrayIconBuilder,
};
use std::fs;
use std::os::windows::process::CommandExt;
use winit::event_loop::{ControlFlow, EventLoopBuilder};
use image;

fn main() {
    if !fs::metadata("./data/cache").is_ok() {
        fs::create_dir("./data/cache").unwrap();
    }
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let icon = load_icon(std::path::Path::new("./data/tools/logo.png"));
    let event_loop = EventLoopBuilder::new().build().unwrap();
    let tray_menu = Menu::new();
    
    let _ = tray_menu.append_items(&[
        &MenuItem::new("quit", true, None)
    ]);

    let mut _tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("lofi tray")
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    let menu_channel = MenuEvent::receiver();
    
    thread::spawn(move || {
        loop {
            let output = Command::new("./data/tools/yt-dlp.exe")
                .arg("--format")
                .arg("bestaudio")
                .arg("--skip-download")
                .arg("--get-url")
                .arg("--no-warnings")
                .arg("--quiet")
                .arg("https://www.youtube.com/watch?v=jfKfPfyJRdk").creation_flags(CREATE_NO_WINDOW).output().unwrap();
            let url = std::str::from_utf8(&output.stdout).unwrap().trim();
    
            let result = reqwest::blocking::get(url).unwrap().text().unwrap();
            let bytes = result.as_bytes();
    
            let parsed = m3u8_rs::parse_playlist_res(&bytes);
            let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
            let sink = rodio::Sink::try_new(&handle).unwrap();
            
            match parsed {
                Ok(Playlist::MasterPlaylist(_pl)) => (),
                Ok(Playlist::MediaPlaylist(pl)) => {
                    let mut i = 0;
                    for segment in &pl.segments {
                        if i == 2 {
                            i = 0;
                        } else {
                            i += 1;
                        }
    
                        let status = Command::new("./data/tools/ffmpeg.exe")
                            .arg("-i")
                            .arg(&segment.uri)
                            .arg("-y")
                            .arg("-hide_banner")
                            .arg("-loglevel")
                            .arg("error")
                            .arg(format!("./data/cache/out{}.mp3",i)).creation_flags(CREATE_NO_WINDOW).status().unwrap();
    
                        if status.success() {
                            let file = std::io::BufReader::new(std::fs::File::open(format!("./data/cache/out{}.mp3",i)).unwrap());
                            sink.append(Decoder::new(file).unwrap());
                        }
    
                        loop {
                            if sink.len() < 3 {
                                break;
                            }
                            thread::sleep(std::time::Duration::from_secs(1));
                        }
                            
                    }
                    
                },
                Err(_) => (),
            }
        }
    });

    let _ = event_loop.run(move |_event, event_loop| {
        event_loop.set_control_flow(ControlFlow::Wait);

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == MenuId("1001".to_string()) {
                exit(0);
            }
        }
    });
    
    fn load_icon(path: &std::path::Path) -> tray_icon::Icon {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::open(path)
                .expect("Failed to open icon path")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
    }
}