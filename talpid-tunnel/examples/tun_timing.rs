//! Measure how long it takes to create the Windows tunnel device.
//!
//! The tunnel provider keeps the wintun adapter alive between connections, and creating that
//! adapter is by far the slowest part of setting up the tunnel device. This measures the
//! difference between reusing the adapter and creating it for every connection.
//!
//! Must be run as administrator, with the directory containing `wintun.dll` as the only argument:
//!
//! ```not_rust
//! cargo run -p talpid-tunnel --example tun_timing -- dist-assets/binaries/x86_64-pc-windows-msvc/wintun
//! ```

#[cfg(not(windows))]
fn main() {
    eprintln!("This example only measures the Windows tunnel device");
}

#[cfg(windows)]
fn main() {
    imp::main()
}

#[cfg(windows)]
mod imp {
    use std::net::{IpAddr, Ipv4Addr};
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{Duration, Instant};

    use talpid_tunnel::tun_provider::{TunConfig, TunProvider, blocking_config};

    const ITERATIONS: usize = 5;

    /// Time to wait between iterations, to let Windows finish tearing the adapter down.
    const SETTLE: Duration = Duration::from_secs(1);

    pub fn main() {
        init_logger();

        let resource_dir = PathBuf::from(
            std::env::args_os()
                .nth(1)
                .expect("expected the directory containing wintun.dll as the first argument"),
        );
        assert!(
            resource_dir.join("wintun.dll").exists(),
            "{} contains no wintun.dll",
            resource_dir.display()
        );

        println!("Creating the tunnel device {ITERATIONS} times, dropping it in between.");
        println!();

        println!("== Adapter destroyed between iterations ==");
        let destroyed = measure(&resource_dir, false);

        println!();
        println!("== Adapter reused between iterations ==");
        let reused = measure(&resource_dir, true);

        println!();
        report("destroyed between iterations", &destroyed);
        report("reused between iterations", &reused);
    }

    fn measure(resource_dir: &Path, reuse_adapter: bool) -> Vec<Duration> {
        let mut provider = TunProvider::new(tun_config(resource_dir));
        let mut times = Vec::with_capacity(ITERATIONS);

        for iteration in 1..=ITERATIONS {
            // Use a different address every iteration, as reconnecting to another relay does, so
            // that reuse has to replace the addresses of the previous connection.
            let address = Ipv4Addr::new(10, 64, 0, 2 + u8::try_from(iteration).unwrap());
            provider.config_mut().addresses = vec![IpAddr::V4(address)];

            let start = Instant::now();
            let tun = provider
                .open_tun()
                .expect("failed to open the tunnel device");
            let elapsed = start.elapsed();

            println!("  {iteration}: {elapsed:?}");
            times.push(elapsed);

            drop(tun);
            if !reuse_adapter {
                provider.close_adapter();
            }
            thread::sleep(SETTLE);
        }

        times
    }

    /// A tunnel config with a single IPv4 address, as a typical tunnel has. Note that adding an
    /// IPv6 address makes the device wait for the IPv6 interface to appear, which never happens
    /// when IPv6 is disabled on the host. The address is replaced for every iteration.
    fn tun_config(resource_dir: &Path) -> TunConfig {
        let mut config = blocking_config(resource_dir.to_path_buf());
        config.addresses = vec![IpAddr::V4(Ipv4Addr::new(10, 64, 0, 2))];
        config
    }

    fn report(label: &str, times: &[Duration]) {
        let mut sorted = times.to_vec();
        sorted.sort_unstable();

        let sum: Duration = sorted.iter().sum();
        println!(
            "{label}: min={:?} median={:?} max={:?} mean={:?}",
            sorted[0],
            sorted[sorted.len() / 2],
            sorted[sorted.len() - 1],
            sum / sorted.len() as u32,
        );
    }

    /// Print log records, so that the timings logged by the tunnel provider and the message
    /// logged by `tun` when it falls back to creating the adapter are visible.
    fn init_logger() {
        struct Logger;

        impl log::Log for Logger {
            fn enabled(&self, _metadata: &log::Metadata<'_>) -> bool {
                true
            }

            fn log(&self, record: &log::Record<'_>) {
                println!("     [{}] {}", record.level(), record.args());
            }

            fn flush(&self) {}
        }

        static LOGGER: Logger = Logger;

        log::set_logger(&LOGGER).expect("failed to install logger");
        log::set_max_level(log::LevelFilter::Debug);
    }
}
