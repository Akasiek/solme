PNPM ?= pnpm
ARCH ?= x86_64
MAKEPKG ?= makepkg
ARCH_PACKAGE_DIR ?= packaging/arch-bin
DEB_PACKAGE = src-tauri/target/release/bundle/deb/solme_0.1.0_amd64.deb

APPIMAGE_ENV = APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1 ARCH=$(ARCH)

.PHONY: build windows deb appimage arch-pkg arch-install install

build:
	$(APPIMAGE_ENV) $(PNPM) tauri build

windows:
	pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/build-windows.ps1

deb:
	$(APPIMAGE_ENV) $(PNPM) tauri build --bundles deb

appimage:
	$(APPIMAGE_ENV) $(PNPM) tauri build --bundles appimage

arch-pkg: deb
	cp $(DEB_PACKAGE) $(ARCH_PACKAGE_DIR)/
	cd $(ARCH_PACKAGE_DIR) && $(MAKEPKG) -Cf

arch-install: deb
	cp $(DEB_PACKAGE) $(ARCH_PACKAGE_DIR)/
	cd $(ARCH_PACKAGE_DIR) && $(MAKEPKG) -Cfsi

install: arch-install
