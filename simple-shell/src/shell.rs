use std::{
    env,
    ffi::CString,
    io::{Write, stdin, stdout},
    os::fd::{IntoRawFd, OwnedFd},
    str::FromStr,
};

use nix::{
    libc::{self},
    sys::wait::waitpid,
    unistd::{ForkResult, close, dup2_raw, execvp},
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

// command: ls
// command: ls > output.txt

pub fn handle_execute(command: &str) {
    let mut commands = command
        .trim()
        .split('|')
        .collect::<Vec<_>>()
        .into_iter()
        .peekable();

    let mut pipes: Vec<Option<OwnedFd>> = vec![];

    while let Some(command) = commands.next() {
        let is_next_command = commands.peek().is_some();

        let (mut read_end, mut write_end) = (None, None);

        if is_next_command {
            let (read, write) = nix::unistd::pipe().expect("error in opening descriptor");
            read_end = Some(read);
            write_end = Some(write);
        };

        // read end from previous
        let read_from_previous = match pipes.pop() {
            Some(r) => r,
            None => None,
        };

        // push the pipe to the vector for the next command
        pipes.push(read_end);

        let (command, redirect) = match command.trim().split_once('>') {
            Some((command, redirect)) => (command, Some(redirect.trim())),
            None => (command, None),
        };

        let command_parts = command.split_ascii_whitespace().collect::<Vec<_>>();
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
                println!(
                    "executing command: {command}: read: {}, write: {}",
                    read_from_previous.is_some(),
                    write_end.is_some()
                );
                execute(command, &args, redirect, read_from_previous, write_end);
            }
        };
    }

    // println!("command executed: {}", cmd_line);
}

fn execute(
    command: &str,
    args: &[CString],
    redirect: Option<&str>,
    read_end: Option<OwnedFd>,
    write_end: Option<OwnedFd>,
) {
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

            // if redirect is present then close the stdout and open the given file path
            if let Some(path) = redirect {
                close(1).expect(
                    "error in closing the stdout in child, which is inherit from the parent",
                );
                let fd = nix::fcntl::open(
                    path,
                    nix::fcntl::OFlag::O_APPEND
                        | nix::fcntl::OFlag::O_CREAT
                        | nix::fcntl::OFlag::O_RDWR,
                    nix::sys::stat::Mode::S_IRWXU,
                )
                .unwrap_or_else(|err| {
                    eprintln!("error occured while opening redirect fd: {}", err);
                    unsafe { libc::exit(1) };
                })
                .into_raw_fd();
                println!("raw-fd: {:?}", fd);
            };

            if let Some(read) = read_end {
                unsafe { dup2_raw(stdin(), read).expect("msg") };
            }

            if let Some(write) = write_end {
                unsafe { dup2_raw(stdout(), write).expect("msg") };
            }

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
