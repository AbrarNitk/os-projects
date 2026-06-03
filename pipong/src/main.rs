use std::{
    fs::File,
    io::{BufRead, BufReader, Write, stdin},
    os::fd::{FromRawFd, IntoRawFd, OwnedFd},
    process::exit,
};

use nix::{
    libc::getpid,
    sys::wait::waitpid,
    unistd::{ForkResult, fork, getppid},
};

fn main() {
    let (p1_read, p1_write) = nix::unistd::pipe().expect("error in pipe open");
    let (p2_read, p2_write) = nix::unistd::pipe().expect("error in pipe open");

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!("from parent: child: {}, with parent: {}", child, unsafe {
                getpid()
            });

            // CRITICAL: Close the ends the parent does not use!
            drop(p1_read);
            drop(p2_write);
            parent(p2_read, p1_write);
            waitpid(child, None).expect("error in waiting for the child in parent");
            println!("child returns in parent");
        }
        Ok(ForkResult::Child) => {
            println!(
                "from child: parent: {}, with child: {}",
                getppid(),
                unsafe { getpid() }
            );

            // CRITICAL: Close the ends the parent does not use!
            drop(p2_read);
            drop(p1_write);
            child(p1_read, p2_write);
        }
        Err(err) => {
            eprintln!("error in forking the child: {}", err);
            exit(1);
        }
    }
}

fn child(reader: OwnedFd, writer: OwnedFd) {
    // unsafe { dup2_raw(&reader, STDIN_FILENO).expect("error") };
    // dup2_stdin(&reader).unwrap();
    // dup2_stdout(&writer).unwrap();

    // This entirely avoids dup2 and the std::io::stdin() trap!
    let reader_file = unsafe { File::from_raw_fd(reader.into_raw_fd()) };
    let mut reader = BufReader::new(reader_file);

    // This entirely avoids dup2 and the std::io::stdin() trap!
    let mut writer_file = unsafe { File::from_raw_fd(writer.into_raw_fd()) };

    // drop(reader);
    // drop(writer);

    let mut buffer = String::new();

    println!("child waiting here");
    while let Ok(bytes_read) = reader.read_line(&mut buffer) {
        if 0 == bytes_read {
            println!("Pipe closed (EOF). Exiting child loop.");
            break;
        }

        let msg = format!("Child-Ack: {}", buffer);

        // send the echo back to the parent
        writer_file.write(msg.as_bytes()).unwrap();
        writer_file.flush().unwrap();

        buffer.clear();
    }
}

fn parent(reader: OwnedFd, writer: OwnedFd) {
    // dup2_stdin(&reader).unwrap();
    // dup2_stdout(&writer).unwrap();

    let mut reader = BufReader::new(unsafe { File::from_raw_fd(reader.into_raw_fd()) });
    let mut writer = unsafe { File::from_raw_fd(writer.into_raw_fd()) };

    let mut buffer = String::new();

    while let Ok(read_bytes) = stdin().read_line(&mut buffer) {
        if 0 == read_bytes {
            println!("parent read 0 bytes from terminal");
            break;
        }

        println!("from-parent::cli: {}", buffer);

        // send message to the child
        writer.write(buffer.as_bytes()).unwrap();
        writer.flush().unwrap();
        buffer.clear();

        // read message from child
        reader
            .read_line(&mut buffer)
            .expect("read error from child in parent");

        println!("parent::message-from-child: {}", buffer);
        buffer.clear();
    }
}
