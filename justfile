# TODO fix
test-skip-hacks := 'false'

here := `pwd`
pkg := here / 'vte'

@def: build

[private]
@list:
  just -l

[private]
@ensure-arena:
  mkdir -p 'local'

[group: "setup"]
setup-pkg:
  #!/usr/bin/env bash
  set -euo pipefail
  to_install=()
  for p in xmlstarlet gobject-introspection glib2-devel meson; do
    if ! pacman -Qi "$p" &>/dev/null; then
        to_install+=("$p")
    fi
  done
  if [[ "${#to_install[@]}" -gt 0 ]]; then
    sudo pacman -S "${to_install[@]}"
  fi

[group: "setup"]
[working-directory: 'gir']
setup-gir-install-bin:
  git submodule update --remote
  cargo install --path .

[group: "setup"]
[working-directory: 'gir-files']
setup-gir-fix-files:
  bash fix.sh

# ==============================================================================

[private]
[confirm]
@_vte-clean:
  true
 
[group: "vte"]
vte-clean:
  if [[ -d 'local/vte-2.91' ]] && just _vte-clean; then \
    rm -rf 'local/vte-2.91'; fi

[group: "vte"]
vte-clone: ensure-arena
  git clone --depth 1 https://gitlab.gnome.org/GNOME/vte 'local/vte-2.91'

[group: "vte"]
vte-clone-maybe:
  if ! [[ -d 'local/vte-2.91' ]]; then \
    just vte-clone; fi

[group: "vte"]
[working-directory: 'local/vte-2.91']
vte-configure:
  meson setup _build -Dvapi=false -Dglade=false --prefix="{{pkg}}"

[group: "vte"]
[working-directory: 'local/vte-2.91']
vte-build:
  ninja -C _build

[group: "vte"]
[working-directory: 'local/vte-2.91']
vte-install:
  ninja -C _build install

# ==============================================================================

[group: "zoha-vte-sys"]
zoha-vte-sys-hack:
  {{ test-skip-hacks }} || sed -Ei \
    '/^\s*PRINT_CONSTANT\((\((gint|guint)\) )?((VTE_SYSTEMD_[A-Z_]+)|VTE_UUID_FORMAT_ANY_ID128|VTE_UUID_FORMAT_ID128)\);$/d' \
    zoha-vte-sys/tests/constant.c
  {{ test-skip-hacks }} || sed -Ei \
    '/^\s*\(("\((gint|guint)\))?\s*((VTE_SYSTEMD_[A-Z_]+)|VTE_UUID_FORMAT_ANY_ID128|VTE_UUID_FORMAT_ID128)", "[0-9]{1,2}"\)(,)?$/d' \
    zoha-vte-sys/tests/abi.rs
  {{ test-skip-hacks }} || sed -Ei \
    '/^\s*\("((VTE_SYSTEMD_[A-Z_]+)|VTE_UUID_FORMAT_ANY_ID128|VTE_UUID_FORMAT_ID128)", "[a-z_-]+"\)(,)?$/d' \
    zoha-vte-sys/tests/abi.rs

[group: "zoha-vte-sys"]
[working-directory: 'zoha-vte-sys']
zoha-vte-sys-gir: 
  PKG_CONFIG_PATH="$(just pkg-conf-path)" \
  gir -o .
  git apply < manual.diff

[private]
@pkg-conf-path-hack:
  # TODO Some features are marked as 0.86, yet the pc is set to 0.85
  sed -Ei \
    's/0\.85\.0/0\.86\.0/g' \
    "vte/lib/pkgconfig/vte-2.91.pc"

[private]
@pkg-conf-path: pkg-conf-path-hack
  echo "{{pkg}}/lib/pkgconfig"

[group: "zoha-vte-sys"]
[working-directory: 'zoha-vte-sys']
zoha-vte-sys-build: 
  PKG_CONFIG_PATH="$(just pkg-conf-path)" \
  cargo build

[group: "zoha-vte-sys"]
[working-directory: 'zoha-vte-sys']
zoha-vte-sys-test: 
  PKG_CONFIG_PATH="$(just pkg-conf-path)" \
  cargo test

# ==============================================================================

[group: "zoha-vte"]
zoha-vte-gen:
  python3 gir/generator.py --no-fmt --gir-files-directories \
    "{{here}}/gir-files/" \
    "{{pkg}}/share/gir-1.0/"

[group: "zoha-vte"]
zoha-vte-patch:
  git apply < manual.diff

[group: "zoha-vte"]
[working-directory: 'zoha-vte-sys']
zoha-vte-unbound:
  gir -o . -m not_bound

# ==============================================================================

[group: "setup"]
setup: \
  setup-pkg \
  setup-gir-install-bin \
  setup-gir-fix-files

[group: "zz_stage"]
stage0-vte: \
  vte-clone-maybe \
  vte-configure \
  vte-build \
  vte-install

[group: "zz_stage"]
stage1-zoha-vte-sys: \
  zoha-vte-sys-gir \
  zoha-vte-sys-hack \
  zoha-vte-sys-build \
  zoha-vte-sys-test

[group: "zz_stage"]
stage2-zoha-vte: \
  zoha-vte-gen \
  zoha-vte-patch \
  zoha-vte-unbound

[group: "zz_stage"]
stage3-fmt:
  cd zoha-vte-sys && cargo fmt
  cd zoha-vte     && cargo fmt

[group: "zz_stage"]
build: \
  stage0-vte \
  stage1-zoha-vte-sys \
  stage2-zoha-vte \
  stage3-fmt

