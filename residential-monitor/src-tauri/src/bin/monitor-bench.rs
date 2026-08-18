use clap::{Parser, Subcommand};
use residential_monitor_lib::bench::{
    analyze_all, binding_evidence, compare_batches, generate_profile, replay_c1, replay_peak,
    verify_design_db,
};
use residential_monitor_lib::evidence::Decision;
use residential_monitor_lib::storage::{hold_uncommitted_bundle, CommitBundle, StorageCoordinator};
use residential_monitor_lib::transport::profiles;
use residential_monitor_lib::workload::WorkloadSpec;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "monitor-bench", about = "C0 性能与能力基准入口")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[arg(long)]
        average_active: u32,
        #[arg(long)]
        days: u32,
        #[arg(long, default_value = "full")]
        profile: String,
        #[arg(long, default_value = "bench-data")]
        out: PathBuf,
    },
    Analyze {
        #[arg(long)]
        all_generated: bool,
        #[arg(long, default_value = "bench-data")]
        dir: PathBuf,
    },
    CompareBatches {
        #[arg(long, default_value = "full")]
        synchronous: String,
        #[arg(long, default_value = "bench-data")]
        dir: PathBuf,
    },
    Replay {
        #[arg(long)]
        active: u32,
        #[arg(long)]
        hz: u32,
        #[arg(long)]
        duration: String,
        #[arg(long, default_value = "peak")]
        profile: String,
        #[arg(long, default_value = "full")]
        synchronous: String,
    },
    BindingCapabilities {
        #[arg(long, default_value = "bench-data")]
        dir: PathBuf,
    },
    VerifyDesignDb {
        #[arg(long)]
        average_active: u32,
        #[arg(long)]
        days: u32,
        #[arg(long, default_value = "c1")]
        profile: String,
        #[arg(long, default_value = "bench-data")]
        dir: PathBuf,
    },
    KillChild {
        #[arg(long)]
        mode: String,
        #[arg(long)]
        db: PathBuf,
        #[arg(long)]
        ready: PathBuf,
    },
    Profiles,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Generate {
            average_active,
            days,
            profile,
            out,
        } => {
            let _ = WorkloadSpec::profile_full(average_active, days);
            assert!(profile == "full" || profile == "smoke");
            let report = generate_profile(&out, average_active, days).expect("generate");
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
        }
        Commands::Analyze { all_generated, dir } => {
            assert!(all_generated, "C0 只提供 --all-generated 分析入口");
            let report = analyze_all(&dir).expect("analyze");
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
        }
        Commands::CompareBatches { synchronous, dir } => {
            assert_eq!(synchronous, "full");
            let report = compare_batches(&dir).expect("compare");
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
        }
        Commands::Replay {
            active,
            hz,
            duration,
            profile,
            synchronous,
        } => {
            assert_eq!(synchronous, "full");
            let parsed = parse_duration(&duration);
            let out = PathBuf::from("bench-data");
            let report = match profile.as_str() {
                "peak" => replay_peak(active, hz, parsed, &out).expect("replay"),
                "c1" => replay_c1(active, hz, parsed, &out).expect("replay"),
                other => panic!("未知 profile {other}"),
            };
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
        }
        Commands::VerifyDesignDb {
            average_active,
            days,
            profile,
            dir,
        } => {
            assert_eq!(profile, "c1");
            let report = verify_design_db(&dir, average_active, days).expect("verify");
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
        }
        Commands::KillChild { mode, db, ready } => {
            run_kill_child(&mode, &db, &ready);
        }
        Commands::BindingCapabilities { dir } => {
            std::fs::create_dir_all(&dir).expect("mkdir");
            let bundle = binding_evidence(&dir).expect("binding");
            assert_ne!(bundle.decision, Decision::Reject);
            let path = dir.join("sqlite-binding.json");
            bundle.write_json(&path).expect("write");
            println!("{}", serde_json::to_string_pretty(&bundle).expect("json"));
        }
        Commands::Profiles => {
            println!(
                "{}",
                serde_json::to_string_pretty(&profiles()).expect("json")
            );
        }
    }
}

fn run_kill_child(mode: &str, db: &std::path::Path, ready: &std::path::Path) {
    let bundle = CommitBundle {
        writer_epoch: 1,
        bundle_seq: 1,
        payload: "1,1,8,16".into(),
    };
    match mode {
        "before-commit" => {
            let _hold = hold_uncommitted_bundle(db, &bundle).expect("hold");
            std::fs::write(ready, b"ready").expect("ready");
            std::thread::sleep(Duration::from_secs(30));
        }
        "after-commit" => {
            let mut coordinator = StorageCoordinator::open(db).expect("open");
            coordinator.commit(&bundle).expect("commit");
            std::fs::write(ready, b"committed").expect("ready");
            std::thread::sleep(Duration::from_secs(30));
        }
        "commit-unknown" => {
            let mut coordinator = StorageCoordinator::open(db).expect("open");
            coordinator.commit(&bundle).expect("commit");
            std::fs::write(ready, b"unknown").expect("ready");
            std::process::exit(0);
        }
        other => panic!("未知 kill mode {other}"),
    }
}

fn parse_duration(value: &str) -> Duration {
    if let Some(minutes) = value.strip_suffix('m') {
        Duration::from_secs(minutes.parse::<u64>().expect("minutes") * 60)
    } else if let Some(seconds) = value.strip_suffix('s') {
        Duration::from_secs(seconds.parse::<u64>().expect("seconds"))
    } else {
        panic!("duration 只接受 30m 或 2s");
    }
}
