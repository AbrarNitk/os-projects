use std::{
    ffi::{CStr, CString},
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

pub fn handle_execute(cmd_line: &str) {
    println!("executing command: {}", cmd_line);
    execute(cmd_line);
    println!("command executed: {}", cmd_line);
}

fn execute(command: &str) {
    match unsafe { nix::unistd::fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!("parent is waiting");
            waitpid(child, None).unwrap();
        }
        Ok(ForkResult::Child) => {
            let command_parts = command.trim().split_ascii_whitespace().collect::<Vec<_>>();
            if command_parts.is_empty() {
                return;
            }

            let args = command_parts
                .iter()
                .map(|string| {
                    CString::from_str(string).unwrap_or_else(|_| unsafe { libc::_exit(1) })
                })
                .collect::<Vec<_>>();

            let cmd = match CString::from_str(&command_parts[0]) {
                Ok(cstring) => cstring,
                Err(_) => {
                    unsafe { libc::_exit(1) };
                }
            };

            execvp::<_>(&cmd.clone(), &args).unwrap();
        }
        Err(err) => {
            panic!("unable to fork the process: {}", err);
        }
    }
}
