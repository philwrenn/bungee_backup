%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: bungee_backup
Summary: Simple service and tray application to manage Restic backups.
Version: @@VERSION@@
Release: 1
License: GPLv3
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/bungee-backup/icons
mkdir -p %{buildroot}/etc
mkdir -p %{buildroot}/lib/systemd/system
mkdir -p %{buildroot}/var/backups/restic
cp %{buildroot}/../../../../../resources/systemd/bungee-backup.service %{buildroot}/lib/systemd/system/
cp %{buildroot}/../../../../../resources/desktop/bungee-backup.desktop %{buildroot}/usr/share/applications/
cp %{buildroot}/../../../../../resources/icons/* %{buildroot}/usr/share/bungee-backup/icons/
cp %{buildroot}/../../../../../resources/default/bungee-backup.yml %{buildroot}/etc/
cp -a * %{buildroot}

%clean
#rm -rf %{buildroot}

%files
/var/backups/restic
/usr/bin/bungee-backup
/lib/systemd/system/bungee-backup.service
/usr/share/applications/bungee-backup.desktop
/usr/share/bungee-backup

%config(noreplace) /etc/bungee-backup.yml

%defattr(-,root,root,-)
%{_bindir}/*
