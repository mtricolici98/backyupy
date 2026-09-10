use sysinfo::Disks;

pub struct Mount {
    pub fs_type: String,
    pub dev: String,
    pub mounted_to: Option<String>,
    pub total_space: u64,
}
pub fn get_avail_mounts() -> Vec<Mount> {
    Disks::new_with_refreshed_list()
        .list()
        .iter()
        .filter_map(|d| {
            if d.is_read_only() || !d.is_removable() {
                return None;
            }
            Some(Mount {
                fs_type: d.file_system().to_str()?.to_owned(),
                dev: d.name().to_str()?.to_owned(),
                mounted_to: d.mount_point().to_str().map(String::from),
                total_space: d.total_space(),
            })
        })
        .collect()
}

