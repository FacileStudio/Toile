# Development

How to run the API locally, test it, and pass the gate before a commit.

## Prerequisites

- Rust 1.98.1 (`apps/api/rust-toolchain.toml`) with `rustfmt` and `clippy`
- Docker, for Postgres 16, and `pg_isready` (psql 16 or later)
- bun 1.4 for the client, `filet` 0.19 or later, `mise` and `lefthook`
- Network access to GitHub: `tronc-rs` (`v0.4.0`) and `porte-rs` (`v0.6.1`) come from their
  public repos by tag

On the shared build host, export `CARGO_TARGET_DIR=/srv/scratch/cache/cargo-target-toile` and
`CARGO_PROFILE_DEV_DEBUG=0` before any cargo command. `apps/client/e2e/stack.ts` reads
`CARGO_TARGET_DIR` to find the binaries.

## Setup

```sh
mise trust && mise install
mise run db-up
```

`db-up` starts `postgres:16-alpine` as `toile_dev_postgres` on `127.0.0.1:5657` (password
`postgres`) with `fsync` off. `mise run db-down` stops it.

## Run

```sh
mise run dev
```

The API listens on `:4000` with no Registre: sign-in routes answer `oidc_enabled: false`. The
client's `bun run dev` proxies `/api` to it.

## Tests

```sh
export TEST_DATABASE_URL='postgres://postgres:postgres@127.0.0.1:5657/postgres?sslmode=disable'
cd apps/api && cargo test --locked
```

Unit tests sit beside the code. Integration tests in `apps/api/tests/` build the real router
against Postgres through `tronc::testdb`, one schema per test prefix:

| File | Covers |
|---|---|
| `authorization.rs` | Every board route for each role, the member of another space and anonymous; personal boards |
| `lifecycle.rs` | Create and list per scope, ordering, deleted boards, bad bodies, probes and the API reference drift test |
| `directory.rs` | porte's spaces conformance suite, snapshot apply, retired spaces, the empty-snapshot refusal, GDPR erase |

Without `TEST_DATABASE_URL` the database tests print a skip banner and pass: read the output.

The client's unit tests run with `cd apps/client && bun run test` (vitest, jsdom).

## End to end

```sh
mise run e2e
```

`scripts/e2e.sh` builds the debug `api` binary and the test-only `e2e_seed` example, builds the
client, then runs `apps/client/e2e/toile.e2e.ts` with `bun test` and `playwright-core` driving
headless Chromium. `e2e/stack.ts` recreates the `toile_e2e` database on the local Postgres
(`E2E_ADMIN_URL`, default port 5657), starts the API on a free port with
`CLIENT_DIR=apps/client/build`, then runs `e2e_seed`, which writes alice (owner), bob (member)
and carol (outsider) and mints each a porte session. The run proves the signed-out landing and
that a member creates a board that survives a reload.

It needs Playwright's Chromium headless shell under `~/.cache/ms-playwright`:
`cd apps/client && bunx playwright-core install chromium-headless-shell`. `check.sh` skips it
with a message when Postgres or that browser is missing, or when `SKIP_E2E` is set.

## The gate

```sh
sh scripts/check.sh
filet check .
```

`check.sh` runs `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and
`cargo test --locked` in `apps/api`, then `bun run check`, `bun run test` and `bun run build` in
`apps/client`, then the end-to-end run when it can. `lefthook` runs it on pre-push. Never raise a
filet limit to pass.

Back to the [documentation index](README.md).

## Native apps

The desktop app (`apps/desktop`, Tauri 2) and the phone apps (`apps/client/android` and
`apps/client/ios`, Capacitor 8) wrap the same static client build. In them, `isApp()` in
`$lib/platform.ts` is true: the client talks to `https://toile.facile.studio` (override with
`VITE_API_ORIGIN` at build time) through `@facile/client/native`, with a bearer token and no CSRF
header.

### Sign-in

porte-rs has no mobile flow, so the apps use its CLI flow, which needs no Registre change:

- **Desktop:** the shell opens a one-shot listener on `127.0.0.1` (`login_listen` in
  `apps/desktop/src/login.rs`), then the system browser at
  `/api/auth/oidc?flow=cli&port=<port>&cli_state=<state>`. Registre's callback redirects to the
  listener with the code, the window checks the state and posts the code to
  `/api/auth/oidc/exchange`.
- **Phone:** the in-app browser opens the same URL without a port. porte shows a one-time code
  (valid 60 s); the person pastes it into the app, which redeems it the same way.

The session token lives in the OS keychain on the desktop (`keyring`: Keychain, Credential
Manager, Secret Service) and in the Android Keystore or iOS Keychain on a phone
(`capacitor-secure-storage-plugin`), never in `localStorage` or Capacitor Preferences. Android
backups are off. The board socket carries the token as the `bearer.<token>` subprotocol
(`socketProtocols()` in `$lib/platform.ts`).

### Desktop

Needs the Tauri Linux libraries (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`,
`libdbus-1-dev`, `patchelf`).

```sh
cd apps/desktop
bun install
bun run tauri dev      # against the client dev server on :5173
bun run tauri build    # builds the client, then the installers for this OS
```

The version is `apps/client/package.json`'s. Every app icon comes from
`apps/desktop/app-icon.svg` (the Toile mark of `$lib/brand.ts` on the dark tile): from
`apps/desktop`, run `bunx @tauri-apps/cli icon app-icon.svg -o icons`, copy the `android/`
mipmaps into `apps/client/android/app/src/main/res` and `ios/AppIcon-512@2x.png` into the iOS
`AppIcon.appiconset`, then delete the `android/`, `ios/`, `Square*` and `StoreLogo` outputs.

### Android

Needs JDK 21, the Android SDK (`ANDROID_HOME`) and Node 22 or later for the Capacitor CLI.

```sh
cd apps/client
bun run build
bunx cap sync android
cd android && ./gradlew assembleDebug
```

The debug APK is `android/app/build/outputs/apk/debug/app-debug.apk`. iOS is committed but built
only on a Mac: `bunx cap sync ios`, then open `ios/App` in Xcode.

### Release

A `desktop-v<version>` tag, equal to `apps/client/package.json`'s version, runs
`.github/workflows/release.yml`: macOS dmg (arm64, x64), Linux deb and AppImage, Windows msi and
the Android APK, published with fixed names so `releases/latest/download/<asset>` never changes.
`workflow_dispatch` is a dry run without a release.

| Secret or variable | What it does |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (secrets) | Sign the updater artifacts |
| `TAURI_UPDATER_PUBKEY` (variable) | The matching public key, baked into the app |
| `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD` (secrets) | Sign the APK |

Without the Tauri key and variable, the desktop apps build without self-update and the release
has no `latest.json`. Without the Android secrets the `android` job fails and nothing is
released, so `releases/latest/download/Toile-android.apk` never breaks. The Android keystore
(alias `toile`, certificate SHA-256 `CF:C0:32:B9:...:16:D8`) is backed up in tiroir as
`TOILE_ANDROID_KEYSTORE_B64`, `TOILE_ANDROID_KEYSTORE_PASSWORD` (also the key password) and
`TOILE_ANDROID_KEY_ALIAS`. Neither key can change once an app ships: installed apps would refuse
the next update.
