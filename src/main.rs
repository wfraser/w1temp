use std::env::args;
use std::process::exit;

enum Args {
    ReadAll,
    ReadThese(Vec<String>),
    Help,
}

fn parse_args() -> Args {
    let mut flags = true;
    let mut names = vec![];
    for arg in args().skip(1) {
        if flags {
            if arg == "--help" || arg == "-h" || arg == "--version" || arg == "-V" {
                return Args::Help;
            } else if arg == "--" {
                flags = false;
                continue;
            }
        }
        names.push(arg);
    }
    if names.is_empty() {
        Args::ReadAll
    } else {
        Args::ReadThese(names)
    }
}

fn main() {
    let sensors = match parse_args() {
        Args::Help => {
            eprintln!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            eprintln!("usage: {} [<sensor...>]", args().next().unwrap());
            exit(1);
        }
        Args::ReadAll => {
            w1temp::enumerate_sensors()
                .unwrap_or_else(|e| {
                    eprintln!("Error enumerating sensors: {}", e);
                    exit(2);
                })
        }
        Args::ReadThese(named) => named,
    };

    let mut all_ok = true;
    for sensor in &sensors {
        all_ok &= read_sensor(&sensor).is_ok();
    }
    if !all_ok {
        exit(2);
    }
}

fn read_sensor(sensor: &str) -> Result<(), ()> {
    match w1temp::read_temperature(sensor) {
        Ok(temp) => {
            println!("{}: {}°C ({}°F)", sensor, temp, temp * 9. / 5. + 32.);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error reading sensor \"{}\": {}", sensor, e);
            Err(())
        }
    }
}
