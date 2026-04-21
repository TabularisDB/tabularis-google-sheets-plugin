set shell := ["bash", "-cu"]

# Build the Rust plugin binary (debug).
build:
    cargo build

# Build the Rust plugin binary (release — what CI publishes).
release:
    cargo build --release

# Install + build UI extensions via Vite + @tabularis/plugin-api.
ui-install:
    pnpm --dir ui install

ui-build:
    pnpm --dir ui build

# Run unit tests.
test:
    cargo test

# Build everything (Rust + UI) and copy into Tabularis's plugin folder
# (Linux).
# On macOS:  ~/Library/Application Support/com.debba.tabularis/plugins/google-sheets/
# On Windows: %APPDATA%\com.debba.tabularis\plugins\google-sheets\
dev-install: build ui-build
    mkdir -p ~/.local/share/tabularis/plugins/google-sheets/ui/dist
    cp target/debug/google-sheets-plugin ~/.local/share/tabularis/plugins/google-sheets/
    cp manifest.json ~/.local/share/tabularis/plugins/google-sheets/
    cp ui/dist/*.js ~/.local/share/tabularis/plugins/google-sheets/ui/dist/
    @echo "Installed to ~/.local/share/tabularis/plugins/google-sheets"
    @echo "Restart Tabularis (or toggle the plugin in Settings) to pick up changes."

release-install: release ui-build
    mkdir -p ~/.local/share/tabularis/plugins/google-sheets/ui/dist
    cp target/release/google-sheets-plugin ~/.local/share/tabularis/plugins/google-sheets/
    cp manifest.json ~/.local/share/tabularis/plugins/google-sheets/
    cp ui/dist/*.js ~/.local/share/tabularis/plugins/google-sheets/ui/dist/
    @echo "Installed release build to ~/.local/share/tabularis/plugins/google-sheets"

# Remove the installed plugin.
uninstall:
    rm -rf ~/.local/share/tabularis/plugins/google-sheets

# Run clippy.
lint:
    cargo clippy --all-targets -- -D warnings

# Format the codebase.
fmt:
    cargo fmt --all
