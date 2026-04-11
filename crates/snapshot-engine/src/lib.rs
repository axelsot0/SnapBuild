pub mod auto_snapshot;
pub mod domain;
pub mod sqlite_persistence;

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SnapshotError {
    Io(std::io::Error),
    Parse(String),
    PathOutsideProject(String),
}

impl From<std::io::Error> for SnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEntry {
    pub path: String,
    pub blob_hash: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotManifest {
    pub snapshot_id: String,
    pub created_at_unix_ms: u128,
    pub entries: Vec<SnapshotEntry>,
}

pub fn create_snapshot(
    project_root: &Path,
    store_root: &Path,
    snapshot_id: &str,
) -> Result<SnapshotManifest, SnapshotError> {
    let objects_dir = store_root.join("objects");
    let snapshots_dir = store_root.join("snapshots");
    fs::create_dir_all(&objects_dir)?;
    fs::create_dir_all(&snapshots_dir)?;

    let mut files = Vec::new();
    collect_files(project_root, project_root, &mut files)?;

    let mut entries = Vec::new();
    for abs_path in files {
        let rel_path = abs_path
            .strip_prefix(project_root)
            .map_err(|_| SnapshotError::PathOutsideProject(abs_path.display().to_string()))?;

        let data = fs::read(&abs_path)?;
        let hash = hash_bytes(&data);
        let blob = blob_path(&objects_dir, &hash);
        if !blob.exists() {
            if let Some(parent) = blob.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(blob, &data)?;
        }

        entries.push(SnapshotEntry {
            path: normalize_path(rel_path),
            blob_hash: hash,
            size_bytes: data.len() as u64,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let manifest = SnapshotManifest {
        snapshot_id: snapshot_id.to_string(),
        created_at_unix_ms: unix_ms(),
        entries,
    };

    write_manifest(&manifest, &snapshots_dir)?;
    Ok(manifest)
}

pub fn read_manifest(manifest_path: &Path) -> Result<SnapshotManifest, SnapshotError> {
    let content = fs::read_to_string(manifest_path)?;
    parse_manifest(&content)
}

pub fn manifest_path(store_root: &Path, snapshot_id: &str) -> PathBuf {
    store_root
        .join("snapshots")
        .join(format!("{snapshot_id}.manifest"))
}

pub fn blob_path(objects_dir: &Path, hash: &str) -> PathBuf {
    let (prefix, suffix) = hash.split_at(2);
    objects_dir.join(prefix).join(suffix)
}

fn write_manifest(manifest: &SnapshotManifest, snapshots_dir: &Path) -> Result<(), SnapshotError> {
    let mut body = format!("{}|{}\n", manifest.snapshot_id, manifest.created_at_unix_ms);
    for entry in &manifest.entries {
        body.push_str(&format!(
            "{}\t{}\t{}\n",
            entry.path, entry.blob_hash, entry.size_bytes
        ));
    }
    fs::write(
        snapshots_dir.join(format!("{}.manifest", manifest.snapshot_id)),
        body,
    )?;
    Ok(())
}

fn parse_manifest(content: &str) -> Result<SnapshotManifest, SnapshotError> {
    let mut lines = content.lines();
    let header = lines
        .next()
        .ok_or_else(|| SnapshotError::Parse("missing manifest header".to_string()))?;
    let mut header_parts = header.split('|');
    let snapshot_id = header_parts
        .next()
        .ok_or_else(|| SnapshotError::Parse("missing snapshot id".to_string()))?
        .to_string();
    let created_at_unix_ms = header_parts
        .next()
        .ok_or_else(|| SnapshotError::Parse("missing timestamp".to_string()))?
        .parse::<u128>()
        .map_err(|_| SnapshotError::Parse("invalid timestamp".to_string()))?;

    let mut entries = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut p = line.split('\t');
        let path = p
            .next()
            .ok_or_else(|| SnapshotError::Parse("invalid entry path".to_string()))?
            .to_string();
        let blob_hash = p
            .next()
            .ok_or_else(|| SnapshotError::Parse("invalid entry hash".to_string()))?
            .to_string();
        let size_bytes = p
            .next()
            .ok_or_else(|| SnapshotError::Parse("invalid entry size".to_string()))?
            .parse::<u64>()
            .map_err(|_| SnapshotError::Parse("invalid entry size number".to_string()))?;
        entries.push(SnapshotEntry {
            path,
            blob_hash,
            size_bytes,
        });
    }

    Ok(SnapshotManifest {
        snapshot_id,
        created_at_unix_ms,
        entries,
    })
}

fn collect_files(root: &Path, current: &Path, out: &mut Vec<PathBuf>) -> Result<(), SnapshotError> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if should_ignore(&file_name) {
            continue;
        }

        if path.is_dir() {
            collect_files(root, &path, out)?;
        } else if path.is_file() {
            if path.starts_with(root) {
                out.push(path);
            }
        }
    }
    Ok(())
}

fn should_ignore(name: &str) -> bool {
    [".git", "node_modules", "build", "dist", ".snapbuild"]
        .iter()
        .any(|n| n == &name)
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn hash_bytes(data: &[u8]) -> String {
    let digest = sha256(data);
    to_hex(&digest)
}

fn sha256(input: &[u8]) -> [u8; 32] {
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h = H0;
    let padded = pad_sha256(input);

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().take(16).enumerate() {
            let j = i * 4;
            *word = u32::from_be_bytes([chunk[j], chunk[j + 1], chunk[j + 2], chunk[j + 3]]);
        }
        for i in 16..64 {
            let s0 = small_sigma0(w[i - 15]);
            let s1 = small_sigma1(w[i - 2]);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut hh = h[7];

        for i in 0..64 {
            let t1 = hh
                .wrapping_add(big_sigma1(e))
                .wrapping_add(ch(e, f, g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let t2 = big_sigma0(a).wrapping_add(maj(a, b, c));

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn pad_sha256(input: &[u8]) -> Vec<u8> {
    let bit_len = (input.len() as u64).wrapping_mul(8);
    let mut padded = input.to_vec();
    padded.push(0x80);

    while (padded.len() + 8) % 64 != 0 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());
    padded
}

fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ ((!x) & z)
}

fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn big_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

fn big_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

fn small_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

fn small_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

pub fn source_fingerprint_for_files(
    project_root: &Path,
    files: &[String],
) -> Result<String, SnapshotError> {
    let mut joined = String::new();
    for rel in files {
        let path = project_root.join(rel);
        if path.is_file() {
            let bytes = fs::read(path)?;
            joined.push_str(rel);
            joined.push('\n');
            joined.push_str(&hash_bytes(&bytes));
            joined.push('\n');
        }
    }
    Ok(hash_bytes(joined.as_bytes()))
}

pub fn dependency_fingerprint(project_root: &Path) -> Result<String, SnapshotError> {
    let lockfiles = [
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "Cargo.lock",
        "poetry.lock",
        "Pipfile.lock",
    ];

    let mut joined = String::new();
    for file in lockfiles {
        let path = project_root.join(file);
        if path.is_file() {
            let bytes = fs::read(path)?;
            joined.push_str(file);
            joined.push('\n');
            joined.push_str(&hash_bytes(&bytes));
            joined.push('\n');
        }
    }

    Ok(hash_bytes(joined.as_bytes()))
}

pub fn detect_git_branch(project_root: &Path) -> Option<String> {
    let head = project_root.join(".git").join("HEAD");
    let content = fs::read_to_string(head).ok()?;
    let trimmed = content.trim();
    let prefix = "ref: refs/heads/";
    if let Some(branch) = trimmed.strip_prefix(prefix) {
        return Some(branch.to_string());
    }
    None
}

fn unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic_for_same_content() {
        let a = hash_bytes(b"snapbuild");
        let b = hash_bytes(b"snapbuild");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_changes_for_different_content() {
        let a = hash_bytes(b"snapbuild-a");
        let b = hash_bytes(b"snapbuild-b");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_is_hex_and_blob_path_splits_prefix() {
        let hash = hash_bytes(b"abc");
        assert_eq!(hash.len(), 64);
        assert!(hash
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));

        let path = blob_path(Path::new("objects"), &hash);
        let rendered = path.to_string_lossy();
        assert!(rendered.contains(&hash[..2]));
        assert!(rendered.contains(&hash[2..]));
    }

    #[test]
    fn computes_dependency_fingerprint_from_lockfiles() {
        let base = std::env::temp_dir().join(format!("snapbuild-deps-{}", unix_ms()));
        fs::create_dir_all(&base).unwrap();
        fs::write(
            base.join("Cargo.lock"),
            "[package]
name='x'
",
        )
        .unwrap();

        let fp = dependency_fingerprint(&base).unwrap();
        assert_eq!(fp.len(), 64);

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn detects_git_branch_from_head_file() {
        let base = std::env::temp_dir().join(format!("snapbuild-git-{}", unix_ms()));
        fs::create_dir_all(base.join(".git")).unwrap();
        fs::write(
            base.join(".git/HEAD"),
            "ref: refs/heads/main
",
        )
        .unwrap();

        let branch = detect_git_branch(&base);
        assert_eq!(branch.as_deref(), Some("main"));

        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn creates_snapshot_and_manifest() {
        let base = std::env::temp_dir().join(format!("snapbuild-test-{}", unix_ms()));
        let project = base.join("project");
        let store = base.join(".snapbuild");
        fs::create_dir_all(project.join("src")).unwrap();

        fs::write(project.join("a.txt"), "hello").unwrap();
        fs::write(project.join("src/main.js"), "console.log('x')").unwrap();

        let snapshot = create_snapshot(&project, &store, "snap-1").unwrap();
        assert_eq!(snapshot.entries.len(), 2);

        let manifest = read_manifest(&manifest_path(&store, "snap-1")).unwrap();
        assert_eq!(manifest.snapshot_id, "snap-1");

        for entry in manifest.entries {
            let blob = blob_path(&store.join("objects"), &entry.blob_hash);
            assert!(blob.exists());
        }

        let _ = fs::remove_dir_all(base);
    }
}
