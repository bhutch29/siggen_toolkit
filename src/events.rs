use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Provider {
    pub name: String,
    pub guid: Option<String>,
    pub event_source_name: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct TimeCreated {
    pub system_time: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct System {
    pub provider: Provider,
    pub time_created: TimeCreated,
    pub level: u16,

    #[serde(rename = "EventID")]
    pub event_id: u16,

    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UserData {
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct EventData {
    data: Option<Vec<Data>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MyEvent {
    pub system: System,
    pub event_data: Option<EventData>,
    pub user_data: Option<UserData>
}

#[cfg(windows)]
use win_event_log::prelude::*;

#[cfg(windows)]
pub fn print_event_stuff() {
    println!("Events Stuff!");
    match get_events() {
        Ok(events) => {
            for event in events {
                println!();
                println!("{}", serde_json::to_string_pretty(&event).unwrap());
                println!("------------");
                println!("{:?}", event);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[cfg(windows)]
pub fn get_events() -> Result<Vec<MyEvent>, String> {
    let conditions = vec![
        Condition::or(vec![
            // Condition::filter(EventFilter::level(1, Comparison::Equal)), // Critical
            Condition::filter(EventFilter::level(2, Comparison::Equal)), // Error
            // Condition::filter(EventFilter::level(3, Comparison::Equal)), // Warn
            // Condition::filter(EventFilter::level(4, Comparison::Equal)) // Info
        ])
    ];
    let query = QueryList::new()
        .with_query(
            Query::new()
                .item(
                    QueryItem::selector("Application".to_owned())
                        .system_conditions(Condition::or(conditions))
                        .build(),
                )
                .query(),
        )
        .build();

    WinEvents::get(query).map(|events| {
        let mut my_events = Vec::new();
        for event in events {
            let parsed: MyEvent = event.into(); // TODO: improve failure handling
            my_events.push(parsed);
        }
        my_events
    })
}

#[cfg(not(windows))]
pub fn print_event_stuff() {
    println!("Not supported on Linux");
}

#[cfg(not(windows))]
pub fn get_events() -> Result<Vec<MyEvent>, String> {
    Err("Not supported on Linux".to_string())
}
