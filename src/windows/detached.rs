//! A window in a **separate process**, started by an action of the app and independent of it.
//!
//! A window opened with `iced::window::open` ([`Child`](super::Child)) belongs to the process
//! that opened it: it goes when that process exits. [`Detached`] is the other kind: it starts
//! another program (usually this executable, with arguments that make it open the window you
//! want) in its own process group, with nothing attached to the parent. Closing the main window,
//! or quitting the parent, leaves it running.
//!
//! ```ignore
//! // On a message from a dialog or a button: the app decides what the new process shows.
//! Detached::this_exe()?.arg("--settings").spawn()?;
//! // or, as a task, answered with the pid:
//! Detached::this_exe()?.arg("--settings").task(Message::Spawned)
//! ```
//!
//! The child starts with stdin closed and output discarded ([`inherit_output`](Detached::inherit_output)
//! keeps it for debugging). The parent reaps it when it ends, so it never lingers as a zombie
//! while the parent runs.

use iced::Task;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Another process to start, apart from this one. See the [module](self).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detached {
    program: PathBuf,
    args: Vec<OsString>,
    envs: Vec<(OsString, OsString)>,
    inherit_output: bool,
}

impl Detached {
    /// Starts `program`.
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            envs: Vec::new(),
            inherit_output: false,
        }
    }
    /// Starts this executable again.
    pub fn this_exe() -> io::Result<Self> {
        std::env::current_exe().map(Self::new)
    }
    pub fn arg(mut self, a: impl Into<OsString>) -> Self {
        self.args.push(a.into());
        self
    }
    pub fn args(mut self, a: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        self.args.extend(a.into_iter().map(Into::into));
        self
    }
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }
    /// Keeps the child's stdout and stderr on this process's (default: discarded).
    pub fn inherit_output(mut self, on: bool) -> Self {
        self.inherit_output = on;
        self
    }

    /// The command this starts, as it will run: its own process group, no stdin.
    pub fn command(&self) -> Command {
        let mut c = Command::new(&self.program);
        c.args(&self.args)
            .envs(self.envs.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::null());
        if !self.inherit_output {
            c.stdout(Stdio::null()).stderr(Stdio::null());
        }
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            // Its own process group: a signal sent to this one (Ctrl+C in its terminal, the
            // terminal closing) is not sent to the child.
            c.process_group(0);
        }
        c
    }

    /// Starts the process and returns its pid. The child outlives this process.
    pub fn spawn(&self) -> io::Result<u32> {
        let mut child = self.command().spawn()?;
        let pid = child.id();
        // Reap it when it ends, without ever signalling it.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(pid)
    }

    /// [`spawn`](Self::spawn) as a task, answered with the pid or the error.
    pub fn task<M: Send + 'static>(
        self,
        on_done: impl Fn(io::Result<u32>) -> M + Send + 'static,
    ) -> Task<M> {
        Task::future(async move { on_done(self.spawn()) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_carries_the_program_arguments_and_environment() {
        let d = Detached::new("gallery")
            .arg("--appshell")
            .args(["--size", "1100x720"])
            .env("EXAMPLE_SETTING", "example");
        let c = d.command();
        assert_eq!(c.get_program(), "gallery");
        let args: Vec<_> = c.get_args().collect();
        assert_eq!(args, ["--appshell", "--size", "1100x720"]);
        let envs: Vec<_> = c.get_envs().collect();
        assert_eq!(envs.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn it_starts_a_process_that_is_not_waited_for() {
        let pid = Detached::new("sh")
            .args(["-c", "exit 0"])
            .spawn()
            .expect("sh starts");
        assert!(pid > 0);
        assert!(Detached::new("/nonexistent/program").spawn().is_err());
    }

    #[test]
    fn this_executable_can_be_started_again() {
        let d = Detached::this_exe().unwrap();
        assert!(d.program.is_absolute());
    }
}
