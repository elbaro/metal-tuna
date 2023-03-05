use std::collections::HashSet;

use clap::Parser;
use colored::*;

#[derive(clap::Subcommand)]
enum Command {
    Guide,
}

#[derive(clap::Parser)]
#[command(version, about)]
struct Args {
    #[clap(long)]
    pid: Option<u64>,
    #[clap(long)]
    verbose: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

fn check(name: &str, value: impl Into<Option<bool>>) {
    let value = value.into();
    println!(
        "{} {}",
        match value {
            Some(true) => "✔".green(),
            Some(false) => "✗".red(),
            None => "?".yellow(),
        },
        name,
    );
}

fn power_states() {
    println!("{}", "Power & Frequency Control".bold());
    check("intel_pstate or amd-pstate", None);
    check("", None);
    println!();
}

fn numa() {
    println!("{}", "NUMA".bold());
    check("NUMA affinity", None);
    check("NUMA misses", None);
    check("NUMA foreign", None);
    println!();
}

fn mitigations() {
    let cmdline = std::fs::read_to_string("/proc/cmdline").unwrap();
    let params: HashSet<_> = cmdline.split_whitespace().collect();

    println!("{}", "Disable Speculative Execution Mitigations".bold());
    if params.contains("mitigations=off") {
        check("mitigations=off", true);
        check("Spectre v1 + SWAPGS", true);
        check("Spectre v2", true);
        check("Spectre v3/Meltdown", true);
        check("MDS/Zombieload", true);
        check("TSX Asynchronous Abort", true);
    } else {
        check("mitigations=off", false);
        check("Spectre v1 + SWAPGS", params.contains("nospectre_v1"));
        check("Spectre v2", params.contains("nospectre_v2"));
        check("Spectre v3/Meltdown", params.contains("pti=off"));
        check("MDS/Zombieload", params.contains("mds=off"));
        check(
            "TSX Asynchronous Abort",
            params.contains("tsx_async_abort=off"),
        );
    }
    println!();
}

fn network() {
    let lsmod = std::fs::read_to_string("/proc/modules").unwrap();
    let modules: HashSet<_> = lsmod
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .collect();

    println!("{}", "Disable Iptables".bold());
    check("ip_tables", !modules.contains("ip_tables")); // modprove -rb ip_tables
    check("ip6_tables", !modules.contains("ip6_tables"));
    check("arp_tables", modules.contains("arp_tables"));
    check("ebtables", modules.contains("ebtables"));
    println!();
}

fn perfect_locality(pid: Option<u64>) {
    println!("{}", "Perfect Locality".bold());

    if let Some(pid) = pid {
        let taskset = std::process::Command::new("taskset")
            .arg("-p")
            .arg(pid.to_string())
            .output()
            .unwrap();
        let stdout = std::str::from_utf8(&taskset.stdout).unwrap();
        let mut mask = 0;
        for line in stdout.lines() {
            if line.contains("current affinity mask:") {
                let hex = line.rsplit(' ').next().unwrap();
                mask = u64::from_str_radix(hex, 16).unwrap();
                break;
            }
        }
        let mut num_cpus = 0;
        while mask > 0 {
            num_cpus += 1;
            mask &= mask - 1;
        }
        check("CPU Pinning", num_cpus == 1); // https://www.redhat.com/sysadmin/tune-linux-tips
    }
    check("SO_ATTACH_REUSEPORT_CBPF", None);
    check("RSS: Receive Side Scailing - disable irqbalance", None);
    check("RSS: Receive Side Scailing - set affinity", None);

    let interface = default_network_interface();
    let mut xps = false;
    for i in 0.. {
        let path =
            std::path::PathBuf::from(format!("/sys/class/net/{interface}/queues/tx-{i}/xps_cpus"));
        let Ok(bitmap) = std::fs::read_to_string(path) else {break};
        if bitmap.trim_end().chars().any(|x| x != '0') {
            xps = true;
            break;
        }
    }

    check("XPS: Transmit Packet Steering", xps);
    check("Use the NUMA node that PCIe NIC is attached to", None);

    println!();
}

fn syscall_audit() {
    println!("{}", "Disable Syscall Auditing".bold());
    println!();
}

fn default_network_interface() -> String {
    let route = std::process::Command::new("route").output().unwrap();
    for line in std::str::from_utf8(&route.stdout).unwrap().lines() {
        if line.starts_with("default ") {
            return line.rsplit_once(' ').unwrap().1.to_string();
        }
    }
    unreachable!()
}

fn interrupt_optimizations() {
    let interface = default_network_interface();

    let ethtool = std::process::Command::new("ethtool")
        .arg("-c")
        .arg(&interface)
        .output()
        .unwrap();

    let mut adaptive_rx = false;
    let mut tx_usecs = false;
    for line in std::str::from_utf8(&ethtool.stdout).unwrap().lines() {
        if line.starts_with("Adaptive RX: on") {
            adaptive_rx = true;
        } else if let Some(stripped) = line.strip_prefix("tx-usecs: ") {
            let Ok(num) = stripped.parse::<u32>() else {continue};
            if num >= 256 {
                tx_usecs = true;
            }
        }
    }

    println!("{} - {}", "Interrupt Moderation".bold(), interface);
    check("adaptive-rx or high rx-usecs", adaptive_rx);
    check("high tx-usecs", tx_usecs);
    println!();
}

fn busy_polling() {
    let output = std::process::Command::new("sysctl")
        .arg("net.core.busy_poll")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap().trim_end();

    println!("{}", "Busy Polling".bold());
    check("net.core.busy_poll=1", stdout == "net.core.busy_poll = 1");
    check("SO_ATTACH_REUSEPORT_CBPF", None);
    println!();
}

// fn _spin_locks() {
//     println!("{}", "Spin Locks".bold());
//     check("noqueue qdisk", None);
//     println!();
// }

fn disable_dhcp_after_boot() {
    let ss = std::process::Command::new("/usr/bin/ss")
        .arg("--packet")
        .arg("--processes")
        .output()
        .unwrap();
    let mut has_dhclient = false;
    for line in std::str::from_utf8(&ss.stdout).unwrap().lines() {
        if line.contains("dhclient") {
            has_dhclient = true;
            break;
        }
    }

    let output = std::process::Command::new("ip")
        .arg("addr")
        .arg("show")
        .arg(default_network_interface())
        .output()
        .unwrap();
    let mut forever = true;
    for line in std::str::from_utf8(&output.stdout).unwrap().lines() {
        if line.contains("sec") {
            forever = false;
            break;
        }
    }

    println!("{}", "Disable DHCP".bold());
    check("No DHCP client after boot", !has_dhclient);
    check("Set the lifetime of IP to forever", forever);
    println!();
}

fn others() {
    println!("{}", "Options That Might Not Work".bold());
    check("Disabling Generic Received Offload", None);
    check("Change TCP congestion control algorithm", None);
    check("Transparent huge pages", None);
    println!();
}

fn main() {
    let args = Args::parse();

    match args.command {
        None => {
            power_states();
            numa();
            mitigations();
            network();
            perfect_locality(args.pid);
            syscall_audit();
            interrupt_optimizations();
            busy_polling();
            disable_dhcp_after_boot();
            // _spin_locks();
            others();

            println!("Use --help to get help.");
        }
        Some(Command::Guide) => {
            termimad::print_text(include_str!("../guide.md"));
        }
    }
}
