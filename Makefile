PNPM ?= pnpm
ARCH ?= x86_64
MAKEPKG ?= makepkg
ARCH_PACKAGE_DIR ?= packaging/arch-bin

APPIMAGE_ENV = APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1 ARCH=$(ARCH)

.PHONY: build appimage arch-pkg arch-install install

build:
	$(APPIMAGE_ENV) $(PNPM) tauri build

appimage:
	$(APPIMAGE_ENV) $(PNPM) tauri build --bundles appimage

arch-pkg: build
	cd $(ARCH_PACKAGE_DIR) && $(MAKEPKG) -f

arch-install: build
	cd $(ARCH_PACKAGE_DIR) && $(MAKEPKG) -si

install: arch-install
