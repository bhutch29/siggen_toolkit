#[cfg(target_os="windows")]
use serde::Deserialize;

#[cfg(target_os="windows")]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
struct Provider {
    pub name: String,
    pub guid: Option<String>,
    pub event_source_name: Option<String>,
}

#[cfg(target_os="windows")]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
struct TimeCreated {
    pub system_time: String,
}

#[cfg(target_os="windows")]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
struct System {
    pub provider: Provider,
    pub time_created: TimeCreated,
    pub level: u16,

    #[serde(rename = "EventID")]
    pub event_id: u16,

    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[cfg(target_os="windows")]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
struct Data {
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[cfg(target_os="windows")]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
struct EventData {
    data: Option<Vec<Data>>,
}

#[cfg(target_os="windows")]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
struct MyEvent {
    pub system: System,
    pub event_data: EventData,
}

#[cfg(target_os="windows")]
use win_event_log::prelude::*;

#[cfg(target_os="windows")]
pub fn event_stuff() {
    println!("Events Stuff!");
    let conditions = vec![
        // Condition::filter(EventFilter::level(1, Comparison::Equal)),
        Condition::filter(EventFilter::level(4, Comparison::GreaterThanOrEqual)),
        // Condition::filter(EventFilter::provider("SigGen"))
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

    match WinEvents::get(query) {
        Ok(events) => {
            for event in events {
                println!();
                println!("{}", event);
                let parsed: MyEvent = event.into();
                println!("{:?}", parsed);
                // break;
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[cfg(not(target_os="windows"))]
pub fn event_stuff() {
    println!("Not supported on Linux");
}

