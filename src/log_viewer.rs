use std::sync::{Arc, Mutex};
use eframe::epi::RepaintSignal;
use strum::Display;

#[derive(PartialEq, Display)]
pub enum Source {
    Stdin,
    File
}

#[derive(Debug, Display)]
pub enum Level {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    Trace
}

#[derive(Debug)]
pub struct Item {
    pub time: String,
    pub level: Level,
    pub logger: String,
    pub msg: String,
}

#[derive(Default, Debug)]
pub struct Data {
    pub items: Vec<Item>
}

impl Data {
    pub fn push(&mut self, data: String) {
        self.items.push(Item {
            time: String::new(),
            level: Level::Critical,
            logger: String::from("TODO"),
            msg: data,
        })
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

pub fn watch_stdin(data: Arc<Mutex<Data>>, repaint: Arc<dyn RepaintSignal>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        loop {
            let mut buffer = String::new();
            if let Ok(..) = stdin.read_line(&mut buffer)
            {
                if !buffer.is_empty() {
                    repaint.request_repaint();
                    data.lock().unwrap().push(buffer);
                }
            }
        }
    });
}