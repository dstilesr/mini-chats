mod messages;
mod settings;

use messages::ClientMessage;
use serde_json;
use settings::AppSettings;

fn main() {
    let settings = AppSettings::new();

    println!("Application Settings: {settings:?}");

    let pub_msg = ClientMessage::Publish {
        channel_name: String::from("publish_channel"),
        content: String::from("Something interesting"),
    };

    let sub_msg = ClientMessage::Subscribe {
        channel_name: String::from("publish_channel"),
    };
    let pub_str = serde_json::to_string(&pub_msg).unwrap();
    let sub_str = serde_json::to_string(&sub_msg).unwrap();
    println!("Publish Message: {}", pub_str);
    println!("Subscribe Message: {}", sub_str);
}
