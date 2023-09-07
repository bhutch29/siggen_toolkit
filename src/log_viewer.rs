use std::str::FromStr;
use std::sync::{Arc, Mutex};
use anyhow::anyhow;
use chrono::NaiveDateTime;
use eframe::epi;
use strum::{Display, EnumString};

#[derive(PartialEq, Display)]
pub enum Source {
    Stdin,
    File
}

#[derive(Debug, Display, EnumString)]
pub enum Level {
    #[strum(ascii_case_insensitive)]
    Critical,
    #[strum(ascii_case_insensitive)]
    Error,
    #[strum(ascii_case_insensitive)]
    Warning,
    #[strum(ascii_case_insensitive)]
    Info,
    #[strum(ascii_case_insensitive)]
    Debug,
    #[strum(ascii_case_insensitive)]
    Trace
}

#[derive(Debug)]
pub struct Item {
    pub time: NaiveDateTime,
    pub level: Level,
    pub logger: String,
    pub msg: String,
}

impl FromStr for Item {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let start_date = s.find('[').ok_or(anyhow!("date start not found: {}", s))? + 1;
        let end_date = s.find(']').ok_or(anyhow!("date end not found: {}", s))?;
        let date_str = &s[start_date..end_date];
        let datetime = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S%.3f").map_err(|_err| anyhow!("date format incorrect: {}", s))?;

        let remaining = &s[end_date + 1..];

        let start_logger = remaining.find('[').ok_or(anyhow!("logger start not found: {}", s))? + 1;
        let end_logger = remaining.find(']').ok_or(anyhow!("logger end not found: {}", s))?;
        let logger = &remaining[start_logger..end_logger];

        let remaining = &remaining[end_logger + 1..];

        let start_level = remaining.find('[').ok_or(anyhow!("level start not found: {}", s))? + 1;
        let end_level = remaining.find(']').ok_or(anyhow!("level end not found: {}", s))?;
        let level_str = &remaining[start_level..end_level];
        let level = Level::from_str(level_str)?;

        Ok(Self {
            time: datetime,
            level,
            logger: String::from(logger),
            msg: String::from(s),
        })
    }
}

#[derive(Default, Debug)]
pub struct Data {
    pub items: Vec<Item>,
    pub pending_item: Option<Item>
}

impl Data {
    pub fn push(&mut self, data: &str) {
        if self.pending_item.is_none() { // first data
            self.pending_item = Item::from_str(data).ok();
            return;
        }

        match Item::from_str(data) {
            Ok(new_item) => {
                // TODO: analyze Item and populate necessary databases
                self.items.push(self.pending_item.take().unwrap());
                self.pending_item = Some(new_item);
            }
            Err(_) => {
                let mut pending = self.pending_item.take().unwrap();
                pending.msg.push_str(data);
                self.pending_item = Some(pending);
            }
        };
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

pub fn watch_stdin(data: Arc<Mutex<Data>>, frame: epi::Frame) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        loop {
            let mut buffer = String::new();
            if let Ok(..) = stdin.read_line(&mut buffer)
            {
                if !buffer.is_empty() {
                    frame.request_repaint();
                    data.lock().unwrap().push(&buffer);
                }
            }
        }
    });
}
