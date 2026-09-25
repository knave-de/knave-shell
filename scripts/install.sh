#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage: scripts/install.sh [--user|--system|--prefix PATH]

Installs the Rust/wgpu Knave Shell bootstrap binary and README.
The Wayland runtime is still under rewrite and is not replaced by this
bootstrap installer until the compositor-facing surface is complete.
EOF
}

prefix="${HOME}/.local"
while (($# > 0)); do
    case "$1" in
        --user)
            prefix="${HOME}/.local"
            ;;
        --system)
            prefix="/usr/local"
            ;;
        --prefix)
            if (($# < 2)); then
                echo "--prefix requires a path" >&2
                exit 2
            fi
            prefix="$2"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo build --manifest-path "${repo_root}/Cargo.toml" --workspace --release --locked

install_cmd=(install)
if [[ "$prefix" == /usr/* ]] && [[ "$(id -u)" -ne 0 ]]; then
    install_cmd=(sudo install)
fi

"${install_cmd[@]}" -Dm755 "${repo_root}/target/release/knave-shell" \
    "${prefix}/bin/knave-shell"
"${install_cmd[@]}" -Dm644 "${repo_root}/README.md" \
