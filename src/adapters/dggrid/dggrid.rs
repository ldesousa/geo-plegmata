use std::path::PathBuf;

pub struct DggridAdapter {
    pub executable: PathBuf,
    pub workdir: PathBuf,
}

impl DggridAdapter {
    pub fn new(executable: PathBuf, workdir: PathBuf) -> Self {
        Self {
            executable,
            workdir,
        }
    }
}

impl Default for DggridAdapter {
    fn default() -> Self {
        Self {
            executable: PathBuf::from("dggrid"),
            workdir: PathBuf::from("/dev/shm"),
        }
    }
}
