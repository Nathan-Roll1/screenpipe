# Desktop validation

`bun run typecheck` passed. The 37 focused language-support and managed-settings tests passed; logs are adjacent to this file.

`bun run dev:web` loaded the settings route. The browser mock returned null for unhandled native onboarding and audio-device commands. The observed errors were `Cannot read properties of null (reading 'isCompleted')` at `lib/first-run/use-learning-window.ts:77` and `Cannot read properties of null (reading 'some')` when entering Audio & meetings (`recording-settings.tsx:3611`). The relevant mock/consumer code is unchanged by this PR. Picker selection/persistence was therefore not validated in the browser, and no before/after recording is claimed.

The supported native command was started from `apps/screenpipe-app-tauri`:

```sh
HF_HOME=/path/to/staged-hf-cache RUSTUP_TOOLCHAIN=stable bun run test:tauri orukeet
```

The first attempt stopped because sccache was missing. After installing sccache 0.17.0, the second attempt acquired the required machine-wide build queue and started `pre_build.js`. At initial publication it was still downloading the required ffprobe native dependency; native compilation/tests had not completed. No raw Cargo/Tauri or uncached compilation fallback was used.

Native picker/capture, all-day memory and battery checks remain unverified. Automated CLI/model validation is listed in the main README and is distinct from desktop end-to-end validation.
