use std::{cmp::Ordering, fmt, str::FromStr};

/// A platform that artifacts are built for. The names match the npm platform
/// package suffixes (`@metassr/linux-x64-gnu`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    LinuxX64Gnu,
    LinuxArm64Gnu,
    DarwinArm64,
    DarwinX64,
    Win32X64Msvc,
}

impl Target {
    pub const ALL: [Target; 5] = [
        Target::LinuxX64Gnu,
        Target::LinuxArm64Gnu,
        Target::DarwinArm64,
        Target::DarwinX64,
        Target::Win32X64Msvc,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Target::LinuxX64Gnu => "linux-x64-gnu",
            Target::LinuxArm64Gnu => "linux-arm64-gnu",
            Target::DarwinArm64 => "darwin-arm64",
            Target::DarwinX64 => "darwin-x64",
            Target::Win32X64Msvc => "win32-x64-msvc",
        }
    }

    /// The target this binary was compiled for, if artifacts exist for it.
    pub fn current() -> Option<Target> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        match (
            os,
            arch,
            cfg!(target_env = "gnu"),
            cfg!(target_env = "msvc"),
        ) {
            ("linux", "x86_64", true, _) => Some(Target::LinuxX64Gnu),
            ("linux", "aarch64", true, _) => Some(Target::LinuxArm64Gnu),
            ("macos", "aarch64", _, _) => Some(Target::DarwinArm64),
            ("macos", "x86_64", _, _) => Some(Target::DarwinX64),
            ("windows", "x86_64", _, true) => Some(Target::Win32X64Msvc),
            _ => None,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Target::ALL
            .into_iter()
            .find(|t| t.as_str() == s)
            .ok_or_else(|| format!("unknown target `{s}`"))
    }
}

/// The glibc version of the running process, on Linux GNU targets.
pub fn glibc_version() -> Option<DottedVersion> {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        use std::ffi::{c_char, CStr};
        extern "C" {
            fn gnu_get_libc_version() -> *const c_char;
        }
        // SAFETY: glibc returns a pointer to a static, NUL-terminated string.
        let raw = unsafe { CStr::from_ptr(gnu_get_libc_version()) };
        raw.to_str().ok()?.parse().ok()
    }
    #[cfg(not(all(target_os = "linux", target_env = "gnu")))]
    {
        None
    }
}

/// A version made of dot-separated numbers, such as `2.28` or `0.9.24`.
/// Missing trailing parts compare as zero, so `2.28 == 2.28.0`.
#[derive(Debug, Clone)]
pub struct DottedVersion(Vec<u64>);

impl PartialEq for DottedVersion {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for DottedVersion {}

impl DottedVersion {
    fn part(&self, i: usize) -> u64 {
        self.0.get(i).copied().unwrap_or(0)
    }
}

impl FromStr for DottedVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("empty version".into());
        }
        s.split('.')
            .map(|p| p.parse::<u64>().map_err(|_| format!("bad version `{s}`")))
            .collect::<Result<Vec<_>, _>>()
            .map(DottedVersion)
    }
}

impl Ord for DottedVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let len = self.0.len().max(other.0.len());
        (0..len)
            .map(|i| self.part(i).cmp(&other.part(i)))
            .find(|o| o.is_ne())
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for DottedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for DottedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.0.iter().map(u64::to_string).collect();
        f.write_str(&parts.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_names_round_trip() {
        for t in Target::ALL {
            assert_eq!(t.as_str().parse::<Target>().unwrap(), t);
        }
        assert!("linux-x64-musl".parse::<Target>().is_err());
    }

    #[test]
    fn dotted_versions_compare_numerically() {
        let v = |s: &str| s.parse::<DottedVersion>().unwrap();
        assert!(v("2.38") > v("2.28"));
        assert!(v("0.9.24") > v("0.9.3"));
        assert_eq!(v("2.28"), v("2.28.0"));
        assert!("2.x".parse::<DottedVersion>().is_err());
        assert!("".parse::<DottedVersion>().is_err());
    }

    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    #[test]
    fn detects_glibc() {
        let v = glibc_version().expect("glibc version on a gnu target");
        assert!(v >= "2.17".parse().unwrap());
    }
}
