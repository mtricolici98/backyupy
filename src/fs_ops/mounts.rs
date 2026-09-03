pub struct Mount {
    pub fs_type: String,
    pub dev: String,
    pub mounted_to: Option<String>,
}

fn get_avail_mounts() -> Vec<Mount> {
    vec![
        Mount {
            fs_type: String::from("ext4"),
            dev: String::from("/dev/sdd"),
            mounted_to: None,
        },
        Mount {
            fs_type: String::from("ext4"),
            dev: String::from("/dev/sdb"),
            mounted_to: Some(String::from("/mnt/backup")),
        },
        Mount {
            fs_type: String::from("ntfs"),
            dev: String::from("/dev/sdc"),
            mounted_to: Some(String::from("/mnt/steam")),
        },
    ]
}
