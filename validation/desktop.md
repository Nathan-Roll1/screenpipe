# Desktop validation

`bun run typecheck` passed. The 37 focused language-support and managed-settings tests passed; logs are adjacent to this file.

`bun run dev:web` loaded the settings route. The browser mock returned null for unhandled native onboarding and audio-device commands. The observed errors were `Cannot read properties of null (reading 'isCompleted')` at `lib/first-run/use-learning-window.ts:77` and `Cannot read properties of null (reading 'some')` when entering Audio & meetings (`recording-settings.tsx:3611`). The relevant mock/consumer code is unchanged by this PR. Picker selection/persistence was therefore not validated in the browser, and no before/after recording is claimed.

The supported native command was started from `apps/screenpipe-app-tauri`:

```sh
HF_HOME=/path/to/staged-hf-cache RUSTUP_TOOLCHAIN=stable bun run test:tauri orukeet
```

The first attempt stopped because sccache was missing. After installing sccache 0.17.0, the second attempt acquired the required machine-wide build queue and started `pre_build.js`. The attempt was stopped after 16 minutes 30 seconds while still downloading the required ffprobe native dependency; it exited 143 and released the build queue. Native compilation/tests did not run. No raw Cargo/Tauri or uncached compilation fallback was used. [Final native setup log](native-app-check-public.log).

Native picker/capture, all-day memory and battery checks remain unverified. Automated CLI/model validation is listed in the main README and is distinct from desktop end-to-end validation.
