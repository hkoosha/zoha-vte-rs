here := `pwd`
prefix := here / 'vte'


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

[group: "setup"]
clean:
  # /gir/target is spared
  rm -rf ./target

[private]
@pkg-conf-path-hack prefix='../vte':
  # TODO Some features are marked as 0.86, yet the pc is set to 0.85
  @echo sed -Ei \
    's/0\.85\.0/0\.86\.0/g' \
    "vte/lib/pkgconfig/vte-2.91.pc" > /dev/null
  sed -Ei \
    's#^prefix=.*$#prefix={{prefix}}#g' \
    "vte/lib/pkgconfig/vte-2.91.pc"

@pkg-conf-path: pkg-conf-path-hack
  echo "{{prefix}}/lib/pkgconfig"

[private]
_gir dir *args:
  cd '{{ dir }}' && \
  PKG_CONFIG_PATH="$(just pkg-conf-path)" \
  gir {{ args }}

[private]
_apply dir:
  #!/usr/bin/env bash
  set -euo pipefail
  find {{ dir }} -name '*.diff' | sort | while read -r p; do
    git apply < "$p"
  done

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
  meson setup _build -Dvapi=false -Dglade=false --prefix="{{prefix}}"

[group: "vte"]
[working-directory: 'local/vte-2.91']
vte-build:
  ninja -C _build

[group: "vte"]
[working-directory: 'local/vte-2.91']
vte-install:
  ninja -C _build install

[group: "vte"]
vte-hacks:
  just _apply ./patch
  #git apply < patch/p001_vte_hacks_for_properties.diff 
  #git apply < patch/p002_vte_hacks_for_uri.diff
  #git apply < patch/p002_vte_hacks_for_prop_uri.diff

[group: "vte"]
[working-directory: 'local/vte-2.91']
sudo-vte-install prefix='/opt/zoha/vte': vte-clone-maybe
  if ! [[ -d 'local/sudo-vte-2.91' ]]; then \
    git clone --depth 1 https://gitlab.gnome.org/GNOME/vte 'local/sudo-vte-2.91'; fi
  cd local/sudo-vte-2.91 && \
  meson setup _build -Dvapi=false -Dglade=false --prefix='{{prefix}}' && \
  ninja -C _build && \
  sudo ninja -C _build install

# ==============================================================================

[group: "zoha-vte-sys"]
zoha-vte-sys-gir: 
  just _gir zoha-vte-sys -o .

zoha-vte-sys-hacks:
  just _apply zoha-vte-sys/patch
  # git apply < zoha-vte-sys/patch/p001_c_types.diff
  # git apply < zoha-vte-sys/patch/p002_build_rs_hacks.diff
  # git apply < zoha-vte-sys/patch/p003_drop_systemd.diff
  # git apply < zoha-vte-sys/patch/p004_drop_uuid.diff
  # git apply < zoha-vte-sys/patch/p005_fix_back.diff
  echo sed -Ei \
    '/^\s*PRINT_CONSTANT\((\((gint|guint)\) )?((VTE_SYSTEMD_[A-Z_]+)|VTE_UUID_FORMAT_ANY_ID128|VTE_UUID_FORMAT_ID128)\);$/d' \
    zoha-vte-sys/tests/constant.c
  echo sed -Ei \
    '/^\s*\(("\((gint|guint)\))?\s*((VTE_SYSTEMD_[A-Z_]+)|VTE_UUID_FORMAT_ANY_ID128|VTE_UUID_FORMAT_ID128)", "[0-9]{1,2}"\)(,)?$/d' \
    zoha-vte-sys/tests/abi.rs
  echo sed -Ei \
    '/^\s*\("((VTE_SYSTEMD_[A-Z_]+)|VTE_UUID_FORMAT_ANY_ID128|VTE_UUID_FORMAT_ID128)", "[a-z_-]+"\)(,)?$/d' \
    zoha-vte-sys/tests/abi.rs

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

[group: "zoha-vte-sys"]
[working-directory: 'zoha-vte-sys']
zoha-vte-sys-fmt: 
  cargo fmt

# ==============================================================================

[group: "zoha-vte"]
zoha-vte-gen:
  python3 gir/generator.py --no-fmt --gir-files-directories \
    "{{here}}/gir-files/" \
    "{{prefix}}/share/gir-1.0/"

zoha-vte-hacks-0:
  git apply < zoha-vte/patch/p001_rust_syntax_fix.diff 

zoha-vte-hacks-1:
  git apply < zoha-vte/patch/p002_glib_types_fix.diff

[group: "zoha-vte"]
zoha-vte-fmt:
  cargo fmt

[group: "zoha-vte"]
zoha-vte-unbound:
  just _gir zoha-vte -o . -m not_bound

[group: "zoha-vte"]
[working-directory: 'zoha-vte']
zoha-vte-build: 
  PKG_CONFIG_PATH="$(just pkg-conf-path)" \
  cargo build

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
  vte-install \
  vte-hacks

[group: "zz_stage"]
stage1-zoha-vte-sys: \
  zoha-vte-sys-gir \
  zoha-vte-sys-fmt \
  zoha-vte-sys-hacks \
  zoha-vte-sys-build \
  zoha-vte-sys-test

[group: "zz_stage"]
stage2-zoha-vte: \
  zoha-vte-gen \
  zoha-vte-hacks-0 \
  zoha-vte-fmt \
  zoha-vte-hacks-1 \
  zoha-vte-unbound \
  zoha-vte-build

[group: "zz_stage"]
quick-build: \
  stage1-zoha-vte-sys \
  stage2-zoha-vte

[group: "zz_stage"]
build: \
  stage0-vte \
  quick-build
