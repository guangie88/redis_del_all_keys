#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate redis;
extern crate simple_logger;
extern crate structopt;

#[macro_use]
extern crate structopt_derive;
extern crate url;

use redis::{Client, Commands};
use structopt::StructOpt;
use std::process;
use url::Url;

mod errors {
    error_chain! {
        errors {
        }
    }
}

use errors::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "Authentication Management", about = "Program to perform authentication management.")]
struct MainConfig {
    #[structopt(short = "a", long = "addr", default_value = "redis://127.0.0.1:6379", help = "AMQP server address (host + port)")]
    addr: Url,

    #[structopt(short = "l", long = "logconf", help = "Log config file path")]
    log_config_path: Option<String>,
}

fn run() -> Result<()> {
    let config = MainConfig::from_args();

    if let &Some(ref log_config_path) = &config.log_config_path {
        log4rs::init_file(log_config_path, Default::default())
            .chain_err(|| format!("Unable to initialize log4rs logger with the given config file at '{}'", log_config_path))?;
    } else {
        simple_logger::init()
            .chain_err(|| "Unable to initialize default logger")?;
    }
    
    let client = Client::open(config.addr.clone())
        .chain_err(|| format!("Unable to create redis client with address '{}'", config.addr))?;

    let conn = client.get_connection()
        .chain_err(|| "Unable to get redis client connection")?;

    let keys: Vec<String> = conn.keys("*")
        .chain_err(|| "Unable to get all keys from redis server")?;

    for key in keys.into_iter() {
        let del_fn = || -> std::result::Result<_, _> {
            conn.del(&key)
                .map(|_: usize| key.as_str())
                .map_err(|e| format!("Unable to delete key '{}' from redis server: {}", key, e))
        };

        match del_fn() {
            Ok(key) => info!("Deleted '{}' from redis server!", key),
            Err(e) => error!("{}", e),
        }
    }
    
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {
            info!("Program completed!");
            process::exit(0)
        },

        Err(ref e) => {
            error!("Error: {}", e);

            for e in e.iter().skip(1) {
                error!("> Caused by: {}", e);
            }

            process::exit(1);
        },
    }
}