use sysinfo::{ProcessExt, System, SystemExt};
use users::{get_current_uid, get_user_by_uid, User};

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cmd: String,
    pub owner: User,
    pub is_current_user: bool,
}
impl From<i32> for ProcessInfo {
    fn from(pid: i32) -> Self {
        let sys = System::new_all();
        let process = sys.process(pid).unwrap();
        let current_uid = get_current_uid();

        Self {
            pid: process.pid() as i32,
            name: process.name().to_string(),
            cmd: process.cmd().to_owned().join(" "),
            owner: get_user_by_uid(process.uid).unwrap(),
            is_current_user: get_user_by_uid(process.uid).unwrap().uid() == current_uid,
        }
    }
}

/// get process information for pids
pub fn get_process_info(pids: Vec<u32>) -> Vec<ProcessInfo> {
    pids.into_iter()
        .map(|pid| ProcessInfo::from(pid as i32))
        .collect()
}
