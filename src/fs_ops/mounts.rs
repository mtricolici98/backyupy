use std::{collections::HashMap, path::Path};

use lsblk::{BlockDevice, Mount as LsblkMount};

pub struct Mount {
    pub fs_type: String,
    pub dev: String,
    pub mounted_to: Option<String>,
    pub total_space: u64,
}

pub fn get_all_devices() -> anyhow::Result<Vec<Mount>> {
    let devices = BlockDevice::list()?;
    let mount_list = LsblkMount::list()?;

    let mounted: HashMap<String, LsblkMount> = mount_list
        .into_iter()
        .map(|mount| {
            // Convert "/dev/sda1" into "sda1".
            let key = Path::new(&mount.device)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&mount.device)
                .to_owned();

            (key, mount)
        })
        .collect();

    Ok(devices
        .into_iter()
        .filter_map(|device| {
            let mount = mounted.get(&device.name);

            Some(Mount {
                fs_type: mount.map(|m| m.fstype.clone()).unwrap_or_default(),

                dev: device.fullname.to_string_lossy().into_owned(),

                mounted_to: mount.map(|m| m.mountpoint.to_string_lossy().into_owned()),

                total_space: device.capacity().ok().flatten()? * 512,
            })
        })
        .collect())
}
