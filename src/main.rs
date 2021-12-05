use std::fs;
use std::path::Path;
use walkdir::WalkDir;

enum SimulatedChannel {
    MCS15,
    MCS3 { signal_count: u8 },
}

fn serialize_channel(channel: &SimulatedChannel) -> String {
    match channel {
        SimulatedChannel::MCS15 => "hwPlatform=MCS15".to_string(),
        SimulatedChannel::MCS3 { signal_count } => {
            format!("hwPlatform=MCS3;signalCount={}", signal_count)
        }
    }
}

fn serialize_channels(channels: &[SimulatedChannel]) -> String {
    channels.iter().map(|channel| format!("simulated {}\n", serialize_channel(channel))).collect()
}

fn try_read_file(path: &Path) -> Option<String> {
    path.exists()
        .then(|| fs::read_to_string(path).expect("Unable to read temp.txt"))
}

fn main() {
    let temp_path = Path::new("/home/bhutch/projects/SigGenToolkit/temp.txt");
    // open::that(temp_path).expect("Unable to open temp.txt in default editor");

    for entry in WalkDir::new("/home/bhutch/projects/SigGenToolkit") {
        let entry = entry.unwrap();
        println!("{}", entry.path().display());
    }

    let contents = try_read_file(temp_path);
    if let Some(text) = &contents {
        println!("Contents before overwriting:");
        println!("{}", text);
    }

    let after = serialize_channels(&[
        SimulatedChannel::MCS15,
        SimulatedChannel::MCS3 { signal_count: 8 },
    ]);

    let write_file = || fs::write(&temp_path, &after).expect("Unable to write to temp.txt");

    match contents {
        None => {
            println!("Creating temp.txt");
            write_file();
        }
        Some(before) => {
            if before != after {
                println!("Overwriting temp.txt...");
                write_file();
            } else {
                println!("File already contains desired content");
            }
        }
    }
}
