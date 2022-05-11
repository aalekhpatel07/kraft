

pub fn input() -> std::sync::mpsc::Receiver<String> {
    let (sender, receiver) = std::sync::mpsc::channel::<String>();

    std::thread::spawn(move || loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).expect("Couldn't read from stdin.");
        sender.send(buffer).expect("Couldn't send buffer to the channel.");
    });
    receiver
}