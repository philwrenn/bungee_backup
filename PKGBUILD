# Maintainer: Philip Wrenn <philwrenn@gmail.com>
pkgname=bungee_backup
pkgver=0.6.5
pkgrel=1
arch=('x86_64')
pkgdesc="A simple application to manage backups."
license=('GPL-3.0-only')
backup=('etc/bungee-backup.yml')
makedepends=('git' 'rust' 'clang')
source=('https://github.com/philwrenn/bungee_backup.git#branch=main')
md5sums=('SKIP')

build() {
    return 0
}

package() {
    RUSTFLAGS=-Awarnings cargo install --path="${srcdir}/${pkgname}" --root="$pkgdir"

    mkdir "${pkgdir}"/etc
    mkdir "${pkgdir}"/usr
    mkdir -p "${pkgdir}"/usr/lib/systemd/system
    mkdir -p "${pkgdir}"/usr/share/applications

    rm "${pkgdir}"/.crates.toml
    rm "${pkgdir}"/.crates2.json
    mv "${pkgdir}"/bin/bungee_backup "${pkgdir}"/bin/bungee-backup
    mv "${pkgdir}"/bin "${pkgdir}"/usr/bin

    install -Dm644 "${srcdir}/${pkgname}"/resources/systemd/bungee-backup.service "${pkgdir}"/usr/lib/systemd/system/
    install -Dm644 "${srcdir}/${pkgname}"/resources/desktop/bungee-backup.desktop "${pkgdir}"/usr/share/applications/
    install -Dm600 "${srcdir}/${pkgname}"/resources/default/bungee-backup.yml "${pkgdir}"/etc/
}
