use anyhow::{Context, Result};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, Uid, UpdateKind, Users};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub command: String,
    pub uid: Option<Uid>,
}

/// A snapshot of the processes and users needed to describe a set of sockets.
pub struct Processes {
    system: System,
    users: Users,
    current_uid: Option<Uid>,
}

impl Processes {
    /// Load information for the given pids (and the current process) in a single refresh.
    pub fn load(pids: impl IntoIterator<Item = u32>) -> Result<Self> {
        let current_pid = sysinfo::get_current_pid().ok();
        let mut pids: Vec<Pid> = pids
            .into_iter()
            .map(Pid::from_u32)
            .chain(current_pid)
            .collect();
        pids.sort_unstable();
        pids.dedup();

        let mut system = System::new().context("failed to read process information")?;
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&pids),
            true,
            ProcessRefreshKind::nothing()
                .with_user(UpdateKind::OnlyIfNotSet)
                .with_cmd(UpdateKind::OnlyIfNotSet),
        );
        let users = Users::new_with_refreshed_list().context("failed to read user list")?;
        let current_uid = current_pid
            .and_then(|pid| system.process(pid))
            .and_then(|process| process.user_id())
            .cloned();

        Ok(Self {
            system,
            users,
            current_uid,
        })
    }

    pub fn get(&self, pid: u32) -> Option<ProcessInfo> {
        let process = self.system.process(Pid::from_u32(pid))?;
        let command = process
            .cmd()
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        Some(ProcessInfo {
            name: process.name().to_string_lossy().into_owned(),
            command,
            uid: process.user_id().cloned(),
        })
    }

    pub fn current_uid(&self) -> Option<&Uid> {
        self.current_uid.as_ref()
    }

    pub fn is_root(&self) -> bool {
        #[cfg(unix)]
        return self.current_uid.as_ref().is_some_and(|uid| **uid == 0);
        #[cfg(not(unix))]
        return false;
    }

    /// Resolve a user name, falling back to the numeric id where the platform has one.
    pub fn user_name(&self, uid: &Uid) -> String {
        match self.users.get_user_by_id(uid) {
            Some(user) => user.name().to_owned(),
            #[cfg(unix)]
            None => (**uid).to_string(),
            #[cfg(not(unix))]
            None => "?".to_owned(),
        }
    }
}

/// Convert a raw socket owner id into a [`Uid`].
pub fn uid_from_raw(raw: u32) -> Option<Uid> {
    #[cfg(unix)]
    return usize::try_from(raw)
        .ok()
        .and_then(|raw| Uid::try_from(raw).ok());
    #[cfg(not(unix))]
    {
        let _ = raw;
        None
    }
}
