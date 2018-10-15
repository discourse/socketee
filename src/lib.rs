use std::os::unix::net::UnixDatagram;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::ffi::OsStrExt;

pub fn run(config: Config) -> Result<(), String> {
    let srcsock = setup_srcsock(&config.srcpath)?;

    let mut buf = [0; 1024];

    loop {
        let count = srcsock.recv(&mut buf).unwrap_or_else(|err| {
            eprintln!("Failed to read from {}: {}", config.srcpath, err);
            0
        });
        eprintln!("{}", std::ffi::OsStr::from_bytes(&buf[..count]).to_str().unwrap());

        match setup_dstsock(&config.dstpath) {
            Ok(dstsock) => {
                let wr_count = dstsock.send(&buf[..count]).unwrap_or_else(|err| {
                    eprintln!("Failed to write to {}: {}", config.dstpath, err);
                    0
                });
        
                if wr_count != count {
                    eprintln!("Short write!  Expected to write {}, wrote {}", count, wr_count);
                }
        
                match dstsock.shutdown(std::net::Shutdown::Both) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Failed to close {}: {}", config.dstpath, e);
                    }
                };
            },
            Err(e) => {
                eprintln!("Failed to write datagram to {}: {}", config.dstpath, e);
            }
        }
    }
}

pub struct Config {
    pub srcpath: String,
    pub dstpath: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments!");
        }

        let srcpath = args[1].clone();
        let dstpath = args[2].clone();

        Ok(Config { srcpath, dstpath })
    }
}

fn setup_srcsock(path: &String) -> Result<UnixDatagram, String> {
    let sock = loop {
        match UnixDatagram::bind(&path) {
            Ok(sock) => break Ok(sock),
            Err(ref error) if error.kind() == ErrorKind::AlreadyExists || error.kind() == ErrorKind::AddrInUse => {
                eprintln!("socket {} currently exists; deleting", path);
                match std::fs::remove_file(&path) {
                    Ok(_)  => (),
                    Err(e) => {
                        return Err(format!("Unable to delete existing socket {}: {}", path, e));
                    }
                }
            },
            Err(e)   => {
                return Err(format!("Unable to open {}: {}", path, e));
            },
        }
    };

    match std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o666)) {
        Ok(_)  => (),
        Err(e) => {
            return Err(format!("Unable to set {} to 0666: {}", path, e));
        }
    };

    sock
}

fn setup_dstsock(path: &String) -> Result<UnixDatagram, String> {
    let sock = match UnixDatagram::unbound() {
        Ok(sock) => sock,
        Err(e)   => {
            return Err(format!("Unable to create unbound socket: {}", e));
        }
    };
    
    match sock.connect(path) {
        Ok(_) => return Ok(sock),
        Err(e)   => {
            return Err(format!("Unable to connect to {}: {}", path, e));
        }
    }
}
