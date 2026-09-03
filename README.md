# Backyupy

This is supposed to be a small back-up tool that allows selecting files to include and ignore.

The idea is born from my small bash script that I had to call, with an ever increasing amount of files/folders to exclude, as I didn't want to move some files (e.g., projects that I have in git, media that is replaceable, downloads).

Backup itself is done via `rsync` with a future potential for `rclone` to be integrated to handle remote (cloud) back-up.
