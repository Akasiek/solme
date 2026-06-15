PNPM ?= pnpm
ARCH ?= x86_64

APPIMAGE_ENV = APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1 ARCH=$(ARCH)

.PHONY: build appimage

build:
	$(APPIMAGE_ENV) $(PNPM) tauri build

appimage:
	$(APPIMAGE_ENV) $(PNPM) tauri build --bundles appimage
