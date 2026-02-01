use rdev::{listen, Event};

pub fn test_listener() {
    println!("🎯 Starting event listener test...");

    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error);
    }
}

fn callback(event: Event) {
    println!("Event captured: {:?}", event);
}