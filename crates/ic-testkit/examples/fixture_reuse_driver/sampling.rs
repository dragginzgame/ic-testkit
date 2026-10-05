//! Native process-leader RSS observations, independent of workload policy.
//! Linux retains page-based /proc measurements; Darwin reports native ps RSS.

use std::collections::{BTreeMap, BTreeSet};

#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::{
    fs::{self, File},
    io::{self, Read as _},
    path::Path,
};

use super::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Process {
    pub(super) parent: u32,
    pub(super) resident_bytes: u64,
}

pub(super) type Snapshot = BTreeMap<u32, Process>;

pub(super) const fn source() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux-proc-stat"
    } else {
        "macos-ps-rss"
    }
}

#[cfg(any(target_os = "linux", test))]
fn parse_stat(stat: &str, page_bytes: u64) -> Result<Process, Error> {
    // comm can contain spaces and closing parentheses. Remaining fields start
    // at state (3), then PPID (4), and RSS in pages (24).
    let (_, fields) = stat
        .rsplit_once(')')
        .ok_or(Error::InvalidData("missing process command terminator"))?;
    let mut fields = fields.split_whitespace();
    let parent = fields
        .nth(1)
        .and_then(|field| field.parse().ok())
        .ok_or(Error::InvalidData("invalid process parent"))?;
    let pages: u64 = fields
        .nth(19)
        .and_then(|field| field.parse().ok())
        .ok_or(Error::InvalidData("invalid process RSS pages"))?;
    let resident_bytes = pages
        .checked_mul(page_bytes)
        .ok_or(Error::InvalidData("RSS byte count overflow"))?;
    Ok(Process {
        parent,
        resident_bytes,
    })
}

#[cfg(target_os = "linux")]
fn process_snapshot(proc: &Path, page_bytes: u64) -> Result<Snapshot, Error> {
    let mut processes = Snapshot::new();
    for entry in fs::read_dir(proc)? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let stat = (|| -> io::Result<String> {
            let mut stat = String::new();
            File::open(entry.path().join("stat"))?
                .take(64 * 1024)
                .read_to_string(&mut stat)?;
            Ok(stat)
        })();
        match stat {
            Ok(stat) => {
                processes.insert(pid, parse_stat(&stat, page_bytes)?);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
                ) => {}
            Err(error) if error.raw_os_error() == Some(libc::ESRCH) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(processes)
}

#[cfg(any(target_os = "macos", test))]
fn parse_ps(output: &str) -> Result<Snapshot, Error> {
    let mut processes = Snapshot::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split_whitespace();
        let pid = fields
            .next()
            .and_then(|field| field.parse::<u32>().ok())
            .ok_or(Error::InvalidData("invalid ps PID"))?;
        let parent = fields
            .next()
            .and_then(|field| field.parse().ok())
            .ok_or(Error::InvalidData("invalid ps parent"))?;
        let rss: u64 = fields
            .next()
            .and_then(|field| field.parse().ok())
            .ok_or(Error::InvalidData("invalid ps RSS"))?;
        if fields.next().is_some() {
            return Err(Error::InvalidData("unexpected ps fields"));
        }
        // Darwin ps documents RSS in 1024-byte units, not host memory pages.
        let resident_bytes = rss
            .checked_mul(1024)
            .ok_or(Error::InvalidData("ps RSS byte count overflow"))?;
        if processes
            .insert(
                pid,
                Process {
                    parent,
                    resident_bytes,
                },
            )
            .is_some()
        {
            return Err(Error::InvalidData("duplicate process leader in ps output"));
        }
    }
    Ok(processes)
}

pub(super) fn snapshot() -> Result<Snapshot, Error> {
    #[cfg(target_os = "linux")]
    {
        // SAFETY: sysconf reads the native page size and has no pointer arguments.
        let page_bytes = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        let page_bytes = u64::try_from(page_bytes)
            .ok()
            .filter(|value| *value > 0)
            .ok_or(Error::InvalidData("native page size unavailable"))?;
        process_snapshot(Path::new("/proc"), page_bytes)
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/bin/ps")
            .args(["-A", "-o", "pid=,ppid=,rss="])
            .env("LC_ALL", "C")
            .stderr(Stdio::inherit())
            .output()?;
        if !output.status.success() {
            return Err(Error::CommandFailed {
                program: "/bin/ps".into(),
                status: output.status,
            });
        }
        let output = std::str::from_utf8(&output.stdout)
            .map_err(|_| Error::InvalidData("ps output is not UTF-8"))?;
        parse_ps(output)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    Err(Error::InvalidArgument(
        "native sampling requires Linux or macOS".into(),
    ))
}

pub(super) fn tree_rss(root: u32, processes: &Snapshot) -> Result<(u64, usize), Error> {
    if !processes.contains_key(&root) {
        return Ok((0, 0));
    }
    let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (pid, process) in processes {
        children.entry(process.parent).or_default().push(*pid);
    }
    let mut pending = vec![root];
    let mut seen = BTreeSet::new();
    let mut resident = 0_u64;
    while let Some(pid) = pending.pop() {
        if !seen.insert(pid) {
            continue;
        }
        resident = resident
            .checked_add(processes[&pid].resident_bytes)
            .ok_or(Error::InvalidData("process-tree RSS overflow"))?;
        if let Some(children) = children.get(&pid) {
            pending.extend(children);
        }
    }
    Ok((resident, seen.len()))
}

#[cfg(test)]
mod tests {
    use super::{Process, Snapshot, parse_ps, parse_stat, snapshot, tree_rss};
    use crate::fixture_reuse_driver::Error;

    #[test]
    fn tree_counts_runner_and_descendants_but_not_parent_or_sibling() {
        let processes: Snapshot = [
            (1, 0, 10000),
            (10, 1, 100),
            (11, 1, 20000),
            (20, 10, 200),
            (21, 20, 300),
            (22, 10, 400),
        ]
        .into_iter()
        .map(|(pid, parent, resident_bytes)| {
            (
                pid,
                Process {
                    parent,
                    resident_bytes,
                },
            )
        })
        .collect();
        assert_eq!(tree_rss(10, &processes).unwrap(), (1000, 4));
        assert_eq!(tree_rss(999, &processes).unwrap(), (0, 0));
        let cycles = [
            (
                1,
                Process {
                    parent: 2,
                    resident_bytes: 1,
                },
            ),
            (
                2,
                Process {
                    parent: 1,
                    resident_bytes: 2,
                },
            ),
        ]
        .into();
        assert_eq!(tree_rss(1, &cycles).unwrap(), (3, 2));
    }

    #[test]
    fn kernel_stat_with_parentheses_in_comm_preserves_parent_and_rss() {
        let fields = ["S", "42"]
            .into_iter()
            .chain(["0"; 19])
            .chain(["7"])
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            parse_stat(&format!("123 (name with ) parentheses) {fields}"), 4096).unwrap(),
            Process {
                parent: 42,
                resident_bytes: 7 * 4096
            }
        );
        assert!(matches!(
            parse_stat("123 (name) S bad", 4096),
            Err(Error::InvalidData(_))
        ));
    }

    #[test]
    fn darwin_ps_preserves_parents_and_kib_units_and_rejects_bad_rows() {
        let processes = parse_ps(" 10 1 4\n 20 10 8\n 21 20 0\n").unwrap();
        assert_eq!(tree_rss(10, &processes).unwrap(), (12 * 1024, 3));
        for malformed in [
            "10 1",
            "10 1 -1",
            "10 1 18446744073709551615",
            "10 1 1 extra",
            "10 1 1\n10 1 2",
        ] {
            assert!(matches!(parse_ps(malformed), Err(Error::InvalidData(_))));
        }
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn native_snapshot_includes_this_process_once() {
        let processes = snapshot().unwrap();
        let own = processes
            .get(&std::process::id())
            .expect("native sampler observes test process");
        assert!(own.parent > 0);
        assert!(own.resident_bytes > 0);
        assert!(tree_rss(std::process::id(), &processes).unwrap().0 >= own.resident_bytes);
    }
}
