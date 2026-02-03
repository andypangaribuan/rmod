rustupHomeDir="${PROJECT_DIR}/.rustup"
mkdir -p "${rustupHomeDir}"
export RUSTUP_HOME="${rustupHomeDir}"
export LIBRARY_PATH="${LIBRARY_PATH}:${PROJECT_DIR}/.devbox/nix/profile/default/lib"
