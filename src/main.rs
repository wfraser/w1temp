fn main() {
    for sensor in w1temp::enumerate_sensors()
        .unwrap_or_else(|e| panic!("Error enumerating sensors: {}", e))
    {
        match w1temp::read_temperature(&sensor) {
            Ok(temp) => {
                println!("{}: {}°C ({}°F)", sensor, temp, temp * 9. / 5. + 32.);
            }
            Err(e) => {
                eprintln!("Error reading sensor {}: {}", sensor, e);
            }
        }
    }   
}
