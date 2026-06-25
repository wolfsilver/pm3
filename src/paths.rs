use color_eyre::eyre::bail;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct Paths {
    data_dir: PathBuf,
}

impl Paths {
    pub fn new() -> color_eyre::Result<Self> {
        if let Ok(path) = std::env::var("PM3_DATA_DIR") {
            return Ok(Self {
                data_dir: PathBuf::from(path),
            });
        }
        let Some(base) = dirs::data_dir() else {
            bail!("could not determine data directory");
        };
        Ok(Self {
            data_dir: base.join("pm3"),
        })
    }

    pub fn with_base(base: PathBuf) -> Self {
        Self { data_dir: base }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn pid_file(&self) -> PathBuf {
        self.data_dir.join("pm3.pid")
    }

    pub fn socket_file(&self) -> PathBuf {
        self.data_dir.join("pm3.sock")
    }

    pub fn dump_file(&self) -> PathBuf {
        self.data_dir.join("dump.json")
    }

    pub fn port_file(&self) -> PathBuf {
        self.data_dir.join("pm3.port")
    }

    pub fn log_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    pub fn stdout_log(&self, name: &str) -> PathBuf {
        self.data_dir.join("logs").join(format!("{name}-out.log"))
    }

    pub fn stderr_log(&self, name: &str) -> PathBuf {
        self.data_dir.join("logs").join(format!("{name}-err.log"))
    }

    pub fn rotated_stdout_log(&self, name: &str, n: u32) -> PathBuf {
        self.data_dir
            .join("logs")
            .join(format!("{name}-out.log.{n}"))
    }

    pub fn rotated_stderr_log(&self, name: &str, n: u32) -> PathBuf {
        self.data_dir
            .join("logs")
            .join(format!("{name}-err.log.{n}"))
    }

    /// Resolve a log file path. If it's relative, resolve it relative to `cwd` (if set),
    /// otherwise leave it relative (so it resolves against the pm3 daemon's current directory,
    /// typically the directory containing `pm3.toml`).
    pub fn resolve_log_path(&self, log_path: &str, cwd: Option<&str>) -> PathBuf {
        let path = Path::new(log_path);
        if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(dir) = cwd {
            PathBuf::from(dir).join(path)
        } else {
            path.to_path_buf()
        }
    }

    /// Get the stdout log path, preferring custom path from config if set, otherwise default.
    pub fn get_stdout_log(
        &self,
        name: &str,
        custom_path: Option<&str>,
        cwd: Option<&str>,
    ) -> PathBuf {
        custom_path
            .map(|p| self.resolve_log_path(p, cwd))
            .unwrap_or_else(|| self.stdout_log(name))
    }

    /// Get the stderr log path, preferring custom path from config if set, otherwise default.
    pub fn get_stderr_log(
        &self,
        name: &str,
        custom_path: Option<&str>,
        cwd: Option<&str>,
    ) -> PathBuf {
        custom_path
            .map(|p| self.resolve_log_path(p, cwd))
            .unwrap_or_else(|| self.stderr_log(name))
    }

    /// Get rotated stdout log path, following the same directory as the main log file.
    pub fn get_rotated_stdout_log(
        &self,
        name: &str,
        n: u32,
        custom_path: Option<&str>,
        cwd: Option<&str>,
    ) -> PathBuf {
        if let Some(custom) = custom_path {
            let log_path = self.resolve_log_path(custom, cwd);
            // Create rotated filename in the same directory as the custom log
            if let Some(parent) = log_path.parent() {
                if let Some(filename) = log_path.file_name() {
                    let rotated_name = format!("{}.{}", filename.to_string_lossy(), n);
                    return parent.join(rotated_name);
                }
            }
            // Fallback to appending to the path
            let mut rotated = log_path.to_string_lossy().into_owned();
            rotated.push_str(&format!(".{}", n));
            PathBuf::from(rotated)
        } else {
            self.rotated_stdout_log(name, n)
        }
    }

    /// Get rotated stderr log path, following the same directory as the main log file.
    pub fn get_rotated_stderr_log(
        &self,
        name: &str,
        n: u32,
        custom_path: Option<&str>,
        cwd: Option<&str>,
    ) -> PathBuf {
        if let Some(custom) = custom_path {
            let log_path = self.resolve_log_path(custom, cwd);
            // Create rotated filename in the same directory as the custom log
            if let Some(parent) = log_path.parent() {
                if let Some(filename) = log_path.file_name() {
                    let rotated_name = format!("{}.{}", filename.to_string_lossy(), n);
                    return parent.join(rotated_name);
                }
            }
            // Fallback to appending to the path
            let mut rotated = log_path.to_string_lossy().into_owned();
            rotated.push_str(&format!(".{}", n));
            PathBuf::from(rotated)
        } else {
            self.rotated_stderr_log(name, n)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn test_data_dir_macos() {
        let paths = Paths::new().unwrap();
        let data_dir = paths.data_dir().to_str().unwrap();
        assert!(
            data_dir.ends_with("Library/Application Support/pm3"),
            "expected macOS data dir, got: {data_dir}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_data_dir_linux() {
        let paths = Paths::new().unwrap();
        let data_dir = paths.data_dir().to_str().unwrap();
        assert!(
            data_dir.ends_with(".local/share/pm3") || data_dir.contains("pm3"),
            "expected Linux data dir, got: {data_dir}"
        );
    }

    #[test]
    fn test_pid_file_under_data_dir() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let pid = paths.pid_file();
        assert!(pid.starts_with(paths.data_dir()));
        assert!(pid.ends_with("pm3.pid"));
    }

    #[test]
    fn test_socket_file_under_data_dir() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let sock = paths.socket_file();
        assert!(sock.starts_with(paths.data_dir()));
        assert!(sock.ends_with("pm3.sock"));
    }

    #[test]
    fn test_dump_file_under_data_dir() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let dump = paths.dump_file();
        assert!(dump.starts_with(paths.data_dir()));
        assert!(dump.ends_with("dump.json"));
    }

    #[test]
    fn test_log_dir_under_data_dir() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_dir = paths.log_dir();
        assert!(log_dir.starts_with(paths.data_dir()));
        assert!(log_dir.ends_with("logs"));
    }

    #[test]
    fn test_stdout_log_includes_name() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log = paths.stdout_log("web");
        assert!(log.ends_with("logs/web-out.log"));
    }

    #[test]
    fn test_stderr_log_includes_name() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log = paths.stderr_log("web");
        assert!(log.ends_with("logs/web-err.log"));
    }

    #[test]
    fn test_rotated_stdout_log_format() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        assert!(
            paths
                .rotated_stdout_log("web", 1)
                .ends_with("logs/web-out.log.1")
        );
        assert!(
            paths
                .rotated_stdout_log("web", 2)
                .ends_with("logs/web-out.log.2")
        );
        assert!(
            paths
                .rotated_stdout_log("web", 3)
                .ends_with("logs/web-out.log.3")
        );
    }

    #[test]
    fn test_rotated_stderr_log_format() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        assert!(
            paths
                .rotated_stderr_log("web", 1)
                .ends_with("logs/web-err.log.1")
        );
        assert!(
            paths
                .rotated_stderr_log("web", 2)
                .ends_with("logs/web-err.log.2")
        );
        assert!(
            paths
                .rotated_stderr_log("web", 3)
                .ends_with("logs/web-err.log.3")
        );
    }

    #[test]
    fn test_resolve_log_path_absolute() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let resolved = paths.resolve_log_path("/var/log/app.log", None);
        assert_eq!(resolved, PathBuf::from("/var/log/app.log"));
    }

    #[test]
    fn test_resolve_log_path_relative_with_cwd() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let resolved = paths.resolve_log_path("logs/app.log", Some("/app"));
        assert_eq!(resolved, PathBuf::from("/app/logs/app.log"));
    }

    #[test]
    fn test_resolve_log_path_relative_without_cwd() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let resolved = paths.resolve_log_path("logs/app.log", None);
        assert_eq!(resolved, PathBuf::from("logs/app.log"));
    }

    #[test]
    fn test_get_stdout_log_custom_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_stdout_log("web", Some("./logs/custom.log"), Some("/app"));
        assert_eq!(log_path, PathBuf::from("/app/./logs/custom.log"));
    }

    #[test]
    fn test_get_stdout_log_default_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_stdout_log("web", None, None);
        assert!(log_path.ends_with("logs/web-out.log"));
    }

    #[test]
    fn test_get_stderr_log_custom_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_stderr_log("worker", Some("/var/log/worker-err.log"), None);
        assert_eq!(log_path, PathBuf::from("/var/log/worker-err.log"));
    }

    #[test]
    fn test_get_stderr_log_default_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_stderr_log("worker", None, None);
        assert!(log_path.ends_with("logs/worker-err.log"));
    }

    #[test]
    fn test_get_rotated_stdout_log_custom_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_rotated_stdout_log("app", 1, Some("logs/app.log"), None);
        assert!(log_path.ends_with("logs/app.log.1"));
    }

    #[test]
    fn test_get_rotated_stdout_log_default_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_rotated_stdout_log("app", 1, None, None);
        assert!(log_path.ends_with("logs/app-out.log.1"));
    }

    #[test]
    fn test_get_rotated_stderr_log_custom_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_rotated_stderr_log("svc", 2, Some("/var/log/svc.err"), None);
        assert_eq!(log_path, PathBuf::from("/var/log/svc.err.2"));
    }

    #[test]
    fn test_get_rotated_stderr_log_default_path() {
        let paths = Paths::with_base(PathBuf::from("/tmp/pm3-test"));
        let log_path = paths.get_rotated_stderr_log("svc", 3, None, None);
        assert!(log_path.ends_with("logs/svc-err.log.3"));
    }
}
