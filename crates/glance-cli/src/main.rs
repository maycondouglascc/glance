//! `glance-tree`: print the grouped process tree (M0 spike / debugging aid).
//!
//! Usage: `glance-tree [-c] [-a] [--sort cpu|mem|count|name] [--pss] [--bench N]`

use std::time::{Duration, Instant};

use glance_group::{AppGroup, DesktopIndex, Grouper, Section, format};
use glance_proc::Scanner;

struct Opts {
    children: bool,
    all: bool,
    sort: String,
    pss: bool,
    bench: Option<u32>,
}

fn parse_args() -> Opts {
    let mut o = Opts { children: false, all: false, sort: "cpu".into(), pss: false, bench: None };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "-c" | "--children" => o.children = true,
            "-a" | "--all" => o.all = true,
            "--pss" => o.pss = true,
            "--sort" => o.sort = args.next().unwrap_or_default(),
            "--bench" => o.bench = args.next().and_then(|n| n.parse().ok()),
            "-h" | "--help" => {
                println!("glance-tree [-c|--children] [-a|--all] [--sort cpu|mem|count|name] [--pss] [--bench N]");
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }
    o
}

fn main() {
    let o = parse_args();

    let t = Instant::now();
    let index = DesktopIndex::load();
    let t_index = t.elapsed();

    let mut scanner = Scanner::new().expect("cannot open /proc");
    let first = scanner.scan().expect("scan failed");

    if let Some(n) = o.bench {
        bench(&mut scanner, index, n);
        return;
    }

    std::thread::sleep(Duration::from_millis(1000));
    let report = scanner.scan().expect("scan failed");
    if o.pss {
        let pids: Vec<i32> = scanner.entries().keys().copied().collect();
        scanner.refresh_pss(pids);
    }

    let t = Instant::now();
    let mut grouper = Grouper::new(index, scanner.own_uid());
    let mut groups = grouper.group(scanner.entries());
    let t_group = t.elapsed();

    groups.sort_by(|a, b| {
        a.section.cmp(&b.section).then_with(|| match o.sort.as_str() {
            "mem" => b.mem_kib.cmp(&a.mem_kib),
            "count" => b.members.len().cmp(&a.members.len()),
            "name" => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            _ => b.cpu_permille.cmp(&a.cpu_permille).then(b.mem_kib.cmp(&a.mem_kib)),
        })
    });

    let mut section = None;
    for g in &groups {
        if !o.all && g.section > Section::Apps && !o.children {
            // Default view: apps in full, other sections as one summary line.
        }
        if section != Some(g.section) {
            section = Some(g.section);
            let in_sec: Vec<&AppGroup> = groups.iter().filter(|x| x.section == g.section).collect();
            let cpu: u32 = in_sec.iter().map(|x| x.cpu_permille).sum();
            let mem: u64 = in_sec.iter().map(|x| x.mem_kib).sum();
            let procs: usize = in_sec.iter().map(|x| x.members.len()).sum();
            println!(
                "\n{:<44} {:>4} {:>7} {:>9}",
                format!("{} ({} groups)", g.section.title().to_uppercase(), in_sec.len()),
                procs,
                format::cpu(cpu),
                format::mem(mem)
            );
        }
        if !o.all && g.section != Section::Apps {
            continue;
        }
        print_group(g, &grouper, &scanner, o.children);
    }

    let tot = scanner.totals();
    println!(
        "\nCPU {}  ·  Memory {} / {}  ·  {} processes, {} groups",
        format::cpu(u32::from(tot.cpu_permille)),
        format::mem(tot.mem_total_kib - tot.mem_available_kib),
        format::mem(tot.mem_total_kib),
        tot.process_count,
        groups.len()
    );
    println!(
        "index {} entries in {:?} · first scan {:?} · scan {:?} · group {:?}",
        grouper.index().len(),
        t_index,
        first.duration,
        report.duration,
        t_group
    );
}

fn print_group(g: &AppGroup, grouper: &Grouper, scanner: &Scanner, children: bool) {
    let marker = if children { "▾" } else { "▸" };
    let badge = g.badge.map(|b| format!(" [{b}]")).unwrap_or_default();
    let approx = if g.mem_is_pss { "" } else { "≈" };
    println!(
        "{marker} {:<42} {:>4} {:>7} {:>9}",
        truncate(&format!("{}{badge}", g.name), 42),
        g.members.len(),
        format::cpu(g.cpu_permille),
        format!("{approx}{}", format::mem(g.mem_kib)),
    );
    if !children {
        return;
    }
    let mut members: Vec<_> = g.members.iter().filter_map(|k| scanner.get(k.pid)).collect();
    members.sort_by(|a, b| b.cpu_permille.cmp(&a.cpu_permille).then(b.sample.rss_kib.cmp(&a.sample.rss_kib)));
    for e in members {
        let label = grouper.label(&e.key).unwrap_or(&e.info.comm);
        println!(
            "    {:<26} {:>7} {:>7} {:>9}  {}",
            truncate(label, 26),
            e.key.pid,
            e.cpu_permille.map_or("—".into(), |c| format::cpu(u32::from(c))),
            format::mem(e.sample.rss_kib),
            truncate(&e.info.display_cmdline(), 60)
        );
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n - 1).collect();
        t.push('…');
        t
    }
}

fn bench(scanner: &mut Scanner, index: DesktopIndex, n: u32) {
    let mut grouper = Grouper::new(index, scanner.own_uid());
    let mut scan_total = Duration::ZERO;
    let mut group_total = Duration::ZERO;
    let mut worst = Duration::ZERO;
    for _ in 0..n {
        let r = scanner.scan().unwrap();
        grouper.forget(&r.removed);
        scan_total += r.duration;
        worst = worst.max(r.duration);
        let t = Instant::now();
        std::hint::black_box(grouper.group(scanner.entries()));
        group_total += t.elapsed();
    }
    println!(
        "{} processes · {n} iterations · scan avg {:?} (worst {:?}) · group avg {:?}",
        scanner.entries().len(),
        scan_total / n,
        worst,
        group_total / n
    );
}
