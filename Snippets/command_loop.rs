use std::sync::mpsc;
use std::{io, thread};
use std::time::Duration;

// command loop
fn command_loop(manager: &mut CharacterManager, grid: &mut Grid, items: &mut MapItemGrid) {

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    match tx.send(input) {
                        Ok(_) => (),
                        Err(e) => println!("Failed to send input: {}", e),
                    }
                }
                Err(e) => println!("Failed to read line: {}", e),
            }
        }
    });

    loop {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(input) => {
                let commands: Vec<&str> = input.trim().split(";").collect();
                for command in commands {
                    if command.trim().is_empty() {
                        continue;
                    }
                    let command = parse_command(command);
                    execute_command(command, manager, grid, items);
                }
                manager.automate_all(grid);
                println!("Enter command: ");
            }
            Err(_) => { manager.automate_all(grid); }
        }
    }
}

