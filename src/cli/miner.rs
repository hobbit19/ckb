use crate::cli::SentryConfig;
use crate::helper::{require_path_exists, to_absolute_path};
use ckb_chain_spec::{ChainSpec, SpecPath};
use ckb_miner::{Client, Miner, MinerConfig};
use ckb_util::Mutex;
use clap::ArgMatches;
use crossbeam_channel::unbounded;
use dir::Directories;
use logger::{self, Config as LogConfig};
use serde_derive::Deserialize;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

const DEFAULT_CONFIG_PATHS: &[&str] = &["miner.toml", "nodes/miner.toml"];

#[derive(Clone, Debug, Deserialize)]
struct Config {
    pub logger: LogConfig,
    #[serde(flatten)]
    pub miner: MinerConfig,
    pub chain: SpecPath,
    pub data_dir: PathBuf,
    pub sentry: SentryConfig,
}

impl Config {
    fn resolve_paths(&mut self, base: &Path) {
        self.chain = self.chain.expand_path(base);

        if self.data_dir.is_relative() {
            self.data_dir = base.join(&self.data_dir);
        }

        let dirs = Directories::new(&self.data_dir);
        if let Some(ref file) = self.logger.file {
            let path = dirs.join("logs");
            self.logger.file = Some(path.join(file));
        }
    }

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<Error>> {
        let config_str = std::fs::read_to_string(path.as_ref())?;
        let mut config: Self = toml::from_str(&config_str)?;
        config.resolve_paths(path.as_ref().parent().unwrap_or_else(|| {
            eprintln!("Invalid config file path {:?}", path.as_ref());
            ::std::process::exit(1);
        }));
        Ok(config)
    }
}

pub fn miner(matches: &ArgMatches) {
    let config_path = get_config_path(matches);

    let config = Config::read_from_file(config_path).unwrap_or_else(|e| {
        eprintln!("Invalid config file {:?}", e);
        ::std::process::exit(1);
    });

    let _logger_guard = logger::init(config.logger.clone()).expect("Init Logger");
    let _sentry_guard = config.sentry.clone().init();

    let chain_spec = ChainSpec::read_from_file(&config.chain).expect("Load chain spec");

    let (new_work_tx, new_work_rx) = unbounded();

    let work = Arc::new(Mutex::new(None));

    let client = Client::new(Arc::clone(&work), new_work_tx, config.miner);

    let miner = Miner::new(work, chain_spec.pow_engine(), new_work_rx, client.clone());

    thread::Builder::new()
        .name("client".to_string())
        .spawn(move || client.poll_block_template())
        .expect("Start client failed!");

    miner.run()
}

fn find_default_config_path() -> Option<PathBuf> {
    DEFAULT_CONFIG_PATHS
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
}

pub fn get_config_path(matches: &ArgMatches) -> PathBuf {
    to_absolute_path(
        matches
            .value_of("config")
            .map_or_else(find_default_config_path, |v| {
                require_path_exists(PathBuf::from(v))
            })
            .unwrap_or_else(|| {
                eprintln!("Miner config file not found!");
                ::std::process::exit(1);
            }),
    )
}
