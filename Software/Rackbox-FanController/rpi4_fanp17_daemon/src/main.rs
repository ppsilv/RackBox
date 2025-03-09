use rppal::gpio::Gpio;
use std::fs;
use std::thread;
use std::time::Duration;
use syslog::{Facility, Formatter3164};
use nix::unistd::{fork, ForkResult};
use std::process;
use std::os::unix::io::AsRawFd; // Importação do trait AsRawFd

// Constantes de configuração
const GPIO_FAN_PIN: u8 = 17; // Pino GPIO para controlar o fan
const TEMP_MIN: f32 = 40.0;  // Temperatura mínima para desligar o fan
const TEMP_MAX: f32 = 50.0;  // Temperatura máxima para ligar o fan
const POLL_INTERVAL: u64 = 10; // Intervalo de verificação da temperatura em segundos

fn get_cpu_temperature() -> f32 {
    // Lê a temperatura da CPU do arquivo do sistema
    let temp_str = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .expect("Falha ao ler a temperatura da CPU");
    let temp_millicelsius: f32 = temp_str.trim().parse().expect("Temperatura inválida");
    temp_millicelsius / 1000.0
}

fn log_to_syslog(message: &str) {
    // Configura o logger para syslog
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "rackbox-fancontroller".into(),
        pid: 0,
    };

    match syslog::unix(formatter) {
        Ok(mut logger) => {
            logger.err(message).expect("Falha ao escrever no syslog");
        }
        Err(e) => eprintln!("Falha ao configurar o syslog: {}", e),
    }
}

fn daemonize() {
    // Primeiro fork
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            // Encerra o processo pai
            process::exit(0);
        }
        Ok(ForkResult::Child) => {
            // Continua no processo filho
        }
        Err(_) => {
            eprintln!("Falha ao criar o daemon");
            process::exit(1);
        }
    }

    // Cria uma nova sessão para o processo filho
    nix::unistd::setsid().expect("Falha ao criar uma nova sessão");

    // Segundo fork para garantir que o processo não seja um líder de sessão
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            // Encerra o processo pai
            process::exit(0);
        }
        Ok(ForkResult::Child) => {
            // Continua no processo filho
        }
        Err(_) => {
            eprintln!("Falha ao criar o daemon");
            process::exit(1);
        }
    }

    // Redireciona a saída padrão e de erro para /dev/null
    let dev_null = std::fs::File::open("/dev/null").expect("Falha ao abrir /dev/null");
    nix::unistd::dup2(dev_null.as_raw_fd(), nix::libc::STDIN_FILENO).expect("Falha ao redirecionar stdin");
    nix::unistd::dup2(dev_null.as_raw_fd(), nix::libc::STDOUT_FILENO).expect("Falha ao redirecionar stdout");
    nix::unistd::dup2(dev_null.as_raw_fd(), nix::libc::STDERR_FILENO).expect("Falha ao redirecionar stderr");
}

fn main() {
    // Transforma o processo em um daemon
    daemonize();

    // Inicializa o GPIO após a daemonização
    let gpio = Gpio::new().expect("Falha ao inicializar o GPIO");
    let mut fan_pin = gpio.get(GPIO_FAN_PIN).unwrap().into_output();

    log_to_syslog("Serviço de controle do fan iniciado.");

    loop {
        let temp = get_cpu_temperature();
        println!("Temperatura da CPU: {:.1}°C", temp);

        if temp >= TEMP_MAX && !fan_pin.is_set_high() {
            fan_pin.set_high(); // Liga o fan
            log_to_syslog(&format!("Fan ligado. Temperatura: {:.1}°C", temp));
        } else if temp <= TEMP_MIN && fan_pin.is_set_high() {
            fan_pin.set_low(); // Desliga o fan
            log_to_syslog(&format!("Fan desligado. Temperatura: {:.1}°C", temp));
        }

        thread::sleep(Duration::from_secs(POLL_INTERVAL));
    }
}

