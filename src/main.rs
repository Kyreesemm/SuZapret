use std::env;

use suzapret::{Backend, Result, StrategyStore, ZapretBackend};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("list") => {
            let root = args.next().unwrap_or_else(|| "strategies".into());
            for id in StrategyStore::new(root).list()? { println!("{id}"); }
        }
        Some("start") => {
            let binary_root = args.next().ok_or_else(|| suzapret::SuZapretError::InvalidStrategy("binary root is required".into()))?;
            let strategy_path = args.next().ok_or_else(|| suzapret::SuZapretError::InvalidStrategy("strategy path is required".into()))?;
            let strategy = suzapret::Strategy::from_file(strategy_path)?;
            let mut backend = ZapretBackend::new(binary_root);
            backend.start(&strategy)?;
            println!("Started strategy: {}", strategy.metadata.id);
        }
        Some("stop") => eprintln!("stop requires a long-lived backend owner; use the library API"),
        _ => eprintln!("Usage: suzapret list [strategies] | suzapret start <binary-root> <strategy.toml>"),
    }
    Ok(())
}
