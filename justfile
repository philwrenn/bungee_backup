USER_ID := `id -u`

build:
	cargo build

release:
	cargo build --release

deb:
	cargo deb

rpm:
	cargo rpm build

aur:
	aur build -c

deb-docker:
	mkdir -p ./target
	docker build -t bungee_backup_build:focal -f ./scripts/Dockerfile-focal ./scripts
	./scripts/docker-cmd-focal.sh /root/.cargo/bin/cargo deb
	./scripts/docker-cmd-focal.sh chown -R {{USER_ID}} /bungee_backup/target

rpm-docker:
	mkdir -p ./target
	docker build -t bungee_backup_build:fedora-30 -f ./scripts/Dockerfile-fedora-30 ./scripts
	./scripts/docker-cmd-fedora-30.sh /root/.cargo/bin/cargo rpm build
	./scripts/docker-cmd-fedora-30.sh chown -R {{USER_ID}} /bungee_backup/target
