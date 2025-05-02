use std::{
    env,
    error::Error,
    io::{stdin, stdout, Write},
    path::Path,
    process::{Child, Command, Stdio},
};

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        print!("> "); // rudimentary promptline
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }

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
}
