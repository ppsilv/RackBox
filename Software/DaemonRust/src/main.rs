use rppal::gpio::Gpio;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process;
use syslog::{Facility, Formatter3164, BasicLogger};
use log::{error, info}; // Removida a importação não utilizada `warn`
use libc;
use std::os::fd::AsRawFd;

fn daemonize() {
    // Fork the process
    match unsafe { libc::fork() } {
        -1 => panic!("Failed to fork"),
        0 => (),
        _ => process::exit(0),
    }

    // Create a new session
    if unsafe { libc::setsid() } == -1 {
        panic!("Failed to create a new session");
    }

    // Fork again to ensure the daemon cannot acquire a controlling terminal
    match unsafe { libc::fork() } {
        -1 => panic!("Failed to fork"),
        0 => (),
        _ => process::exit(0),
    }

    // Change the working directory to root
    std::env::set_current_dir("/").expect("Failed to change directory to root");

    // Redirect standard input, output, and error to /dev/null
    let null = std::fs::File::open("/dev/null").expect("Failed to open /dev/null");
    let null_fd = null.as_raw_fd();
    unsafe {
        libc::dup2(null_fd, libc::STDIN_FILENO);
        libc::dup2(null_fd, libc::STDOUT_FILENO);
        libc::dup2(null_fd, libc::STDERR_FILENO);
    }
}

fn setup_logger() -> Result<(), syslog::Error> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_DAEMON,
        hostname: None,
        process: "led_daemon".into(),
        pid: unsafe { libc::getpid().try_into().unwrap() },
    };

    let logger = syslog::unix(formatter)?;
    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(log::LevelFilter::Info))
        .map_err(|e| syslog::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))) // Corrigido
}

fn main() {
    // Daemonize the process
    daemonize();

    // Set up syslog logging
    if let Err(e) = setup_logger() {
        eprintln!("Failed to initialize syslog: {}", e);
        process::exit(1);
    }

    info!("LED daemon started");

    // Initialize GPIO
    let gpio = match Gpio::new() {
        Ok(gpio) => gpio,
        Err(e) => {
            error!("Failed to initialize GPIO: {}", e);
            process::exit(1);
        }
    };

    let mut pin = match gpio.get(14) {
        Ok(pin) => pin.into_output(),
        Err(e) => {
            error!("Failed to get GPIO pin 14: {}", e);
            process::exit(1);
        }
    };

    // Create a flag to control the loop
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Handle Ctrl+C to stop the daemon gracefully
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        info!("Received termination signal, stopping daemon");
    }).expect("Error setting Ctrl+C handler");

    // Blink the LED 8 times per second
    while running.load(Ordering::SeqCst) {
        pin.set_high();
        thread::sleep(Duration::from_millis(62)); // 1/8 of a second

        pin.set_low();
        thread::sleep(Duration::from_millis(62)); // 1/8 of a second
    }

    // Clean up
    pin.set_low();

    info!("LED daemon stopped");
}

