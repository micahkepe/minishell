//! A simple shell implementation in Rust that supports command execution,
//! piping, and changing directories.
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::{
    env,
    error::Error,
    fs,
    path::Path,
    process::{Child, Command, Stdio},
};

/// A simple shell implementation in Rust.
fn main() -> Result<(), Box<dyn Error>> {
    let mut rl = DefaultEditor::new()?;
    let history_path = "/tmp/.minishell_history";

    // Greeting messages
    println!(
        r"
  ___  ____       _     _          _ _
  |  \/  (_)     (_)   | |        | | |
  | .  . |_ _ __  _ ___| |__   ___| | |
  | |\/| | | '_ \| / __| '_ \ / _ \ | |
  | |  | | | | | | \__ \ | | |  __/ | |
  \_|  |_/_|_| |_|_|___/_| |_|\___|_|_|
"
    );
    println!(" Welcome to minishell! Type 'exit' to quit.\n");

    // Load history if it exists, else create it
    match rl.load_history(history_path) {
        Ok(_) => {
            // History loaded successfully
        }
        Err(ReadlineError::Io(_)) => {
            // Create a new history file if it doesn't exist
            fs::File::create(history_path)?;
        }
        Err(err) => {
            eprintln!("minishell: Error loading history: {}", err);
        }
    }

    // read/eval/execute loop
    loop {
        let line = rl.readline("> ");

        match line {
            Ok(line) => {
                let input = line.trim();

                if input.is_empty() {
                    continue;
                }

                // Add the input to history
                rl.add_history_entry(input)?;

                let mut commands = input.trim().split(" | ").peekable();
                let mut prev_stdout = None;
                let mut children: Vec<Child> = Vec::new();

                while let Some(command) = commands.next() {
                    let mut parts = command.split_whitespace();
                    let Some(command) = parts.next() else {
                        continue;
                    };
                    let args = parts;

                    match command {
                        "cd" => {
                            let new_dir = args.peekable().peek().map_or("/", |x| *x);
                            let root = Path::new(new_dir);
                            if let Err(e) = env::set_current_dir(root) {
                                eprintln!("{}", e);
                            }

                            prev_stdout = None;
                        }
                        "exit" => return Ok(()),
                        command => {
                            let stdin = match prev_stdout.take() {
                                Some(output) => Stdio::from(output),
                                None => Stdio::inherit(),
                            };

                            let stdout = if commands.peek().is_some() {
                                Stdio::piped()
                            } else {
                                Stdio::inherit()
                            };

                            let child = Command::new(command)
                                .args(args)
                                .stdin(stdin)
                                .stdout(stdout)
                                .spawn();

                            match child {
                                Ok(mut child) => {
                                    prev_stdout = child.stdout.take();
                                    children.push(child);
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    break;
                                }
                            };
                        }
                    }
                }

                for mut child in children {
                    let _ = child.wait();
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                // Handle Ctrl-C or Ctrl-D gracefully
                println!("\nExiting minishell...");
                rl.save_history(history_path)?;
                break;
            }
            Err(e) => {
                eprintln!("minishell: Error: {:?}", e);
            }
        }
    }

    Ok(())
}
