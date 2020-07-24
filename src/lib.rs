use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;

const BASE_PATH: &str = "/sys/bus/w1/devices";

#[derive(Debug)]
pub enum Error {
    SysFs { msg: String, inner: io::Error },
    InvalidData { msg: String, data: String },
    BadCRC,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SysFs { msg, inner } => {
                write!(f, "error {}: {}", msg, inner)
            }
            Error::InvalidData { msg, data } => {
                write!(f, "sensor data is invalid: {}. (data: {:?})", msg, data)
            }
            Error::BadCRC => {
                f.write_str("sensor data failed CRC check")
            }
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Error::SysFs { ref inner, .. } = self {
            Some(inner)
        } else {
            None
        }
    }
}

pub fn enumerate_sensors() -> Result<Vec<String>, Error> {
    let mut names = vec![];
    for entry in fs::read_dir(&Path::new(BASE_PATH))
        .map_err(|e| Error::SysFs {
            msg: format!("reading directory {:?}", BASE_PATH),
            inner: e,
        })?
    {
        let entry = entry.map_err(|e| Error::SysFs {
            msg: format!("reading directory entry under {:?}", BASE_PATH),
            inner: e,
        })?;
        
        let name = entry.file_name()
            .to_string_lossy() // should always be a no-op
            .into_owned();

        if name.starts_with("28-") {
            names.push(name);
        }
    }
    Ok(names)
}

pub fn read_temperature(device_name: &str) -> Result<f64, Error> {
    let path = Path::new(BASE_PATH).join(device_name).join("w1_slave");
    let file = File::open(&path)
        .map_err(|e| Error::SysFs {
            msg: format!("opening sensor data file {:?}", path),
            inner: e,
        })?;
    parse_file(BufReader::new(file), &path)
}

fn parse_file(f: impl BufRead, path: &Path) -> Result<f64, Error> {
    let mut lines = f.lines();

    let crc_line = lines.next()
        .ok_or(Error::InvalidData { msg: "missing CRC line".into(), data: String::new() })?
        .map_err(|e| Error::SysFs { msg: format!("read error on {:?}", path), inner: e })?;

    let crc_ok = crc_line
        .splitn(12, ' ')
        .nth(11)
        .ok_or_else(|| Error::InvalidData { msg: "CRC line".into(), data: crc_line.clone() })?
        == "YES";

    if !crc_ok {
        return Err(Error::BadCRC);
    }

    let temp_line = lines.next()
        .ok_or(Error::InvalidData { msg: "missing data line".into(), data: String::new() })?
        .map_err(|e| Error::SysFs { msg: format!("read error on {:?}", path), inner: e })?;

    let milli_deg_c = temp_line.splitn(10, ' ')
        .nth(9)
        .ok_or_else(|| Error::InvalidData {
            msg: "wrong number of fields in temperature line".into(),
            data: temp_line.clone(),
        })?
        .splitn(2, '=')
        .nth(1)
        .ok_or_else(|| Error::InvalidData {
            msg: "no '=' in temperature line".into(),
            data: temp_line.clone(),
        })?
        .parse::<f64>()
        .map_err(|e| Error::InvalidData {
            msg: format!("unable to parse temperature as float: {}", e),
            data: temp_line.clone(),
        })?;

    Ok(milli_deg_c / 1000.)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let input = b"60 01 4b 46 7f ff 0c 10 14 : crc=14 YES\n60 01 4b 46 7f ff 0c 10 14 t=22000\n";
        match parse_file(std::io::Cursor::new(&input[..]), Path::new("test data")) {
            Ok(temp) if temp > 21.999 && temp < 22.001 => (),
            Ok(wrong) => panic!("wrong temperature {:?}", wrong),
            Err(e) => panic!("parse failed: {:?}", e),
        }
    }
}
