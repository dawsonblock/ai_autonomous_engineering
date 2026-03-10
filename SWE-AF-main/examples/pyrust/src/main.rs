use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for daemon management commands
    if args.len() > 1 {
        match args[1].as_str() {
            "--daemon" => {
                start_daemon();
                return;
            }
            "--stop-daemon" => {
                stop_daemon();
                return;
            }
            "--daemon-status" => {
                show_daemon_status();
                return;
            }
            "--clear-cache" => {
                clear_cache();
                return;
            }
            _ => {}
        }
    }

    // Check for profiling flags
    let enable_profile = args.contains(&"--profile".to_string());
    let profile_json = args.contains(&"--profile-json".to_string());

    let code = if args.len() > 1 {
        if args[1] == "-c" {
            // Inline code: pyrust -c "print(42)"
            if args.len() < 3 {
                eprintln!("Usage: pyrust -c <code>");
                process::exit(1);
            }
            args[2].clone()
        } else if args[1].starts_with("--") {
            // Handle flag-only invocations
            eprintln!("Usage: pyrust <file.py> | pyrust -c <code> [--profile | --profile-json | --daemon | --stop-daemon | --daemon-status | --clear-cache]");
            process::exit(1);
        } else {
            // File mode: pyrust script.py
            match fs::read_to_string(&args[1]) {
                Ok(contents) => contents,
                Err(e) => {
                    eprintln!("Error reading {}: {}", args[1], e);
                    process::exit(1);
                }
            }
        }
    } else {
        eprintln!("Usage: pyrust <file.py> | pyrust -c <code> [--profile | --profile-json | --daemon | --stop-daemon | --daemon-status | --clear-cache]");
        process::exit(1);
    };

    if enable_profile || profile_json {
        // Execute with profiling (always direct execution, no daemon)
        match pyrust::profiling::execute_python_profiled(&code) {
            Ok((output, profile)) => {
                // Print output first (stdout)
                if !output.is_empty() {
                    print!("{}", output);
                }

                // Print profile (stderr, doesn't interfere with output piping)
                if profile_json {
                    eprintln!("{}", profile.format_json());
                } else {
                    eprintln!("\n{}", profile.format_table());
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    } else {
        // Try daemon execution with fallback to direct execution
        match pyrust::daemon_client::DaemonClient::execute_or_fallback(&code) {
            Ok(output) => {
                if !output.is_empty() {
                    print!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}

/// Start the daemon in background using fork
fn start_daemon() {
    use pyrust::daemon::DaemonServer;

    // Check if daemon is already running
    if pyrust::daemon_client::DaemonClient::is_daemon_running() {
        eprintln!("Daemon is already running");
        process::exit(1);
    }

    // Create a pipe for parent-child synchronization
    let mut pipe_fds: [libc::c_int; 2] = [0, 0];
    unsafe {
        if libc::pipe(pipe_fds.as_mut_ptr()) < 0 {
            eprintln!("Failed to create pipe");
            process::exit(1);
        }
    }
    let pipe_read_fd = pipe_fds[0];
    let pipe_write_fd = pipe_fds[1];

    // Fork the process
    let pid = unsafe { libc::fork() };

    if pid < 0 {
        // Close pipe FDs to prevent resource leak
        unsafe {
            libc::close(pipe_read_fd);
            libc::close(pipe_write_fd);
        }
        eprintln!("Failed to fork process");
        process::exit(1);
    } else if pid > 0 {
        // Parent process - wait for child to signal readiness
        unsafe {
            libc::close(pipe_write_fd); // Close write end in parent
        }

        // Read from pipe to confirm child is ready
        let mut ready_byte = [0u8; 1];
        let result = unsafe {
            libc::read(
                pipe_read_fd,
                ready_byte.as_mut_ptr() as *mut libc::c_void,
                1,
            )
        };

        unsafe {
            libc::close(pipe_read_fd);
        }

        if result == 1 && ready_byte[0] == b'R' {
            // Child signaled ready successfully
            println!("Daemon started with PID {}", pid);
            process::exit(0);
        } else {
            // Child failed to start (pipe closed without sending 'R')
            eprintln!("Failed to start daemon: initialization error");
            process::exit(1);
        }
    }

    // Child process continues below
    unsafe {
        libc::close(pipe_read_fd); // Close read end in child
    }

    // Become session leader
    unsafe {
        if libc::setsid() < 0 {
            // Can still report error via pipe before closing stderr
            let error_msg = b"Failed to create new session\n";
            let _ = libc::write(
                libc::STDERR_FILENO,
                error_msg.as_ptr() as *const libc::c_void,
                error_msg.len(),
            );
            libc::close(pipe_write_fd);
            process::exit(1);
        }
    }

    // Initialize daemon BEFORE closing stderr so errors can be reported
    let daemon = match DaemonServer::new() {
        Ok(d) => d,
        Err(e) => {
            // Report error before closing stderr
            eprintln!("Failed to initialize daemon: {}", e);
            unsafe {
                libc::close(pipe_write_fd);
            }
            process::exit(1);
        }
    };

    // Close standard file descriptors
    unsafe {
        libc::close(0); // stdin
        libc::close(1); // stdout
        libc::close(2); // stderr
    }

    // Redirect standard file descriptors to /dev/null
    unsafe {
        use std::ffi::CString;
        let dev_null = CString::new("/dev/null").unwrap();
        let fd = libc::open(dev_null.as_ptr(), libc::O_RDWR);
        if fd < 0 {
            // Failed to open /dev/null - daemon cannot run safely
            libc::close(pipe_write_fd);
            process::exit(1);
        }
        libc::dup2(fd, 0); // stdin
        libc::dup2(fd, 1); // stdout
        libc::dup2(fd, 2); // stderr
        if fd > 2 {
            libc::close(fd);
        }
    }

    // Signal parent that daemon is ready
    unsafe {
        let ready_byte = b"R";
        libc::write(pipe_write_fd, ready_byte.as_ptr() as *const libc::c_void, 1);
        libc::close(pipe_write_fd);
    }

    // Start the daemon server event loop
    if let Err(_e) = daemon.run() {
        // stderr is now redirected to /dev/null, so errors are lost
        // This is expected for a daemon process
        process::exit(1);
    }
}

/// Stop the running daemon
fn stop_daemon() {
    match pyrust::daemon_client::DaemonClient::stop_daemon() {
        Ok(()) => {
            println!("Daemon stopped successfully");
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Failed to stop daemon: {}", e);
            process::exit(1);
        }
    }
}

/// Show daemon status
fn show_daemon_status() {
    let status = pyrust::daemon_client::DaemonClient::daemon_status();
    println!("{}", status);

    // Exit with 0 if running, 1 if not running
    if pyrust::daemon_client::DaemonClient::is_daemon_running() {
        process::exit(0);
    } else {
        process::exit(1);
    }
}

/// Clear all caches (both global and thread-local)
fn clear_cache() {
    // Clear global cache
    pyrust::clear_global_cache();

    // Clear thread-local cache for current thread
    pyrust::clear_thread_local_cache();

    println!("Cache cleared successfully");
    process::exit(0);
}
