// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-archive.

// substrate-archive is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// substrate-archive is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of // MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with substrate-archive.  If not, see <http://www.gnu.org/licenses/>.

mod cli_opts;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use anyhow::{anyhow, Result};

use node_service::bifrost_runtime::BlockNumber;
use node_service::bifrost_runtime::RuntimeApi;
use sp_runtime::generic;
use sp_runtime::traits::BlakeTwo256;

pub type Block = generic::Block<Header, sp_runtime::OpaqueExtrinsic>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

use substrate_archive::{Archive, ArchiveBuilder, ArchiveConfig, ReadOnlyDb, SecondaryRocksDb};

fn main() -> anyhow::Result<()> {
	let cli = cli_opts::CliOpts::init();
	let config = cli.parse()?;

	let mut archive = run_archive::<SecondaryRocksDb>(&cli.chain_spec, config)?;

	// let mut archive = ArchiveBuilder::<Block, RuntimeApi, SecondaryRocksDb>::with_config(config)
	// 	.chain_spec(Box::new(cli.chain_spec))//Box::new(service::chain_spec::bifrost::chainspec_config(2001.into())))
	// 	.build()?;
	archive.drive()?;

	let running = Arc::new(AtomicBool::new(true));
	let r = running.clone();

	ctrlc::set_handler(move || {
		r.store(false, Ordering::SeqCst);
	})
	.expect("Error setting Ctrl-C handler");
	while running.load(Ordering::SeqCst) {}
	archive.boxed_shutdown()?;
	Ok(())
}

fn run_archive<Db: ReadOnlyDb + 'static>(
	chain_spec: &str,
	config: Option<ArchiveConfig>,
) -> Result<Box<dyn Archive<Block, Db>>> {
	match chain_spec.to_ascii_lowercase().as_str() {
		"bifrost" => {
			let spec =
				node_service::chain_spec::bifrost::ChainSpec::from_json_bytes(&include_bytes!("./bifrost.json")[..],).map_err(|err| anyhow!("{}", err))?;
			let archive =
				ArchiveBuilder::<Block, RuntimeApi, Db>::with_config(config).chain_spec(Box::new(spec)).build()?;
			Ok(Box::new(archive))
		}
		c => Err(anyhow!("unknown chain {}", c)),
	}
}
