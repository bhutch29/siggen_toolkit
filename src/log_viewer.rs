use std::str::FromStr;
use std::sync::{Arc, Mutex};
use chrono::NaiveDateTime;
use eframe::epi::RepaintSignal;
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
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: error handling
        let start_date = s.find('[').unwrap() + 1;
        let end_date = s.find(']').unwrap();
        let date_str = &s[start_date..end_date];
        let datetime = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S%.3f").unwrap();

        let remaining = &s[end_date + 1..];

        let start_logger = remaining.find('[').unwrap() + 1;
        let end_logger = remaining.find(']').unwrap();
        let logger = &remaining[start_logger..end_logger];

        let remaining = &remaining[end_logger + 1..];

        let start_level = remaining.find('[').unwrap() + 1;
        let end_level = remaining.find(']').unwrap();
        let level_str = &remaining[start_level..end_level];
        let level = Level::from_str(level_str).unwrap();

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
    pub items: Vec<Item>
}

impl Data {
    pub fn push(&mut self, data: String) {
        // TODO: Delay emitting until we are sure the previous message is complete, concatenating lines until then
        let item = Item::from_str(data.as_str()).unwrap();
        // TODO: analyze Item and populate necessary databases
        self.items.push(item);
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