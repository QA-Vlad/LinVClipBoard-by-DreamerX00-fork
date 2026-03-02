.PHONY: build build-ui deb install clean check test

build:
	cargo build --release -p clipd -p clipctl

build-ui:
	cd crates/linvclip-ui && npm install && npx tauri build

build-all: build build-ui

check:
	cargo check --workspace
	cargo test -p shared

test:
	cargo test --workspace

deb: build-all
	./packaging/build-deb.sh

install: build
	install -Dm755 target/release/clipd   $(DESTDIR)$(HOME)/.local/bin/clipd
	install -Dm755 target/release/clipctl $(DESTDIR)$(HOME)/.local/bin/clipctl
	install -Dm644 install/clipd.service  $(DESTDIR)$(HOME)/.config/systemd/user/clipd.service
	install -Dm644 install/linvclipboard.desktop $(DESTDIR)$(HOME)/.local/share/applications/linvclipboard.desktop
	systemctl --user daemon-reload
	systemctl --user enable --now clipd.service

clean:
	cargo clean
	rm -rf crates/linvclip-ui/dist crates/linvclip-ui/node_modules
