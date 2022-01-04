# Flatpak

Drg is available as a flatpak package for linux distribution using flatpak.
The latest released version is available on the flathub store.


## Dependencies

`flaptak-builder`

Install the `21.08` versions of flatpak runtime and the rust extension :
```
flatpak install org.freedesktop.Sdk
flatpak install org.freedesktop.Sdk.Extension.rust-stable
```

## Build the flatpak
Generate the dependencies json files from `Carg.lock` using [flatpak-cargo-generator](https://github.com/flatpak/flatpak-builder-tools/blob/master/cargo/flatpak-cargo-generator.py)
```
flatpak-cargo-generator.py ../../Cargo.lock -o generated-sources.json
```
then build the flatpak package :
```
flatpak-builder --install repo io.drogue.drg.yaml --force-clean --user -y
```