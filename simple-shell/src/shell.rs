use std::{
    env,
    ffi::CString,
    io::{Write, stdin, stdout},
    str::FromStr,
};

use nix::{
    libc,
    sys::wait::waitpid,
    unistd::{ForkResult, execvp},
};

pub fn run() {
    loop {
        print!("> ");
        stdout().flush().unwrap();

        let mut command_buffer = String::new();
        stdin()
            .read_line(&mut command_buffer)
            .expect("somethings wrong in reading the command");
        handle_execute(&command_buffer);
        command_buffer.clear();
    }
}

pub fn handle_execute(command: &str) {
    let command_parts = command.trim().split_ascii_whitespace().collect::<Vec<_>>();
    if command_parts.is_empty() {
        return;
    }
    let args = command_parts
        .iter()
        .map(|string| CString::from_str(string).unwrap_or_else(|_| unsafe { libc::_exit(1) }))
        .collect::<Vec<_>>();

    let command = command_parts[0];

    match command {
        "cd" => {
            let args_it = command_parts.iter();
            if let Some(path) = args_it.skip(1).next() {
                let path = std::path::Path::new(path);
                env::set_current_dir(path).expect("error in setting the path");
            }
        }
        "exit" => unsafe {
            libc::exit(0);
        },
        _ => {
            execute(command, &args);
        }
    };

    // println!("command executed: {}", cmd_line);
}

fn execute(command: &str, args: &[CString]) {
    match unsafe { nix::unistd::fork() } {
        Ok(ForkResult::Parent { child }) => {
            // println!("parent is waiting");
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            let cmd = match CString::from_str(&command) {
                Ok(cstring) => cstring,
                Err(_) => {
                    unsafe { libc::_exit(1) };
                }
            };

            // #[warn(irrefutable_let_patterns)]
            if let Err(err) = execvp::<_>(&cmd.clone(), &args) {
                eprintln!("err-in-executing process: {}", err);
            }
        }
        Err(err) => {
            panic!("unable to fork the process: {}", err);
        }
    }
}
