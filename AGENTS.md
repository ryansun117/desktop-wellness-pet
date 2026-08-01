# Wellness Pet contributor guidance

## Architecture boundaries

- Rust owns reminder timing, state transitions, validation, persistence, native integration, and lifecycle behavior.
- React owns rendering, forms, accessible controls, and presentation-only animation state.
- Do not add scheduling timers or reminder truth to TypeScript or browser storage.
- Keep `src-tauri/src/domain` free of Tauri APIs so it remains deterministic and portable.

## Verification

Run these before handing off changes:

```bash
npm run lint
npm test
npm run build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

For native macOS changes, also run `npm run tauri build` and inspect the launched app. Native notifications and launch-at-login can require user permission.

## Project conventions

- Use fixed timestamps in scheduling tests; never make tests sleep.
- Persist meaningful Rust state transitions atomically.
- Treat notification failure as recoverable because the in-app reminder remains available.
- Use typed Tauri commands and a complete startup snapshot; events are incremental freshness signals.
- Preserve the original inline-SVG artwork and avoid remote assets, telemetry, or network services.

