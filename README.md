Bungee Backup
=============

Bungee Backup is a simple tool to configure, run and monitor 
[Restic](https://restic.net/) backups on a Linux desktop.

## Build

Build uses [just](https://github.com/casey/just) command runner.

Build for debugging

`just build`

Build for release

`just release`

Build release deb (local tools and build deps)

`just deb`

Build release rpm (local tools and build deps)

`just rpm`

Build deb using Ubuntu bionic docker image.

`just deb-docker`

Build rpm using Fedora 30 docker image.

`just rpm-docker`

## Configuration

_Currently Restic repos must be initialized manually prior to backing up with Bungee._

Configuration file: `/etc/bungee_backup.yml`

### Backups Parameters

| Parameter         | Required | Description                                                                                                               |
| ----------------- | -------- | ------------------------------------------------------------------------------------------------------------------------- |
| **name**          | Yes      | Backup name                                                                                                               |
| **tag**           | Yes      | Tag used in Restic for this backup                                                                                        |
| **max_age_hours** | Yes      | Once a backup is this old a new snapshot will be taken. (_Currently the only means of scheduling_)                        |
| **env**           | Yes      | Hashmap of [Restic Environment variables.](https://restic.readthedocs.io/en/stable/040_backup.html#environment-variables) |
| **include**       | Yes      | List of paths to include                                                                                                  |
| **exclude**       | No       | List of paths to exclude                                                                                                  |

### Examples

```yaml
- name: Full Backup to B2
  tag: full-backup-to-b2
  max_age_hours: 24
  env:
    B2_ACCOUNT_ID: 000000000000000000
    B2_ACCOUNT_KEY: K000000000000000000000000000
    RESTIC_REPOSITORY: "b2:myreponame"
    RESTIC_PASSWORD: "mysupersecretrepopassword"
  include:
    - /
  exclude:
    - /tmp
    - /proc
    - /dev
    - /sys
    - /run
    - /mnt
    - /media

- name: Projects Backup to sftp
  tag: projects-backup-to-sftp
  max_age_hours: 4
  env:
    RESTIC_REPOSITORY: "sftp:user@host:/srv/restic-repo"
    RESTIC_PASSWORD: "mysupersecretrepopassword"
  include:
    - /path/to/my/projecs
```

## Todos

* Monitoring client is currently CLI-based, replace with tray icon for easy monitoring.
* Automatically initialize Restic repo if it doesn't already exist.