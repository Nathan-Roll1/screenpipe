# Screenpipe Orukeet integration evidence

Measurement artifacts for the optional Orukeet backend, separate from the production PR diff.

- Screenpipe base: `527f5585f81bca226d9d5bb38f62dc0ef6ae650f`
- Integration: `dfa7a1fe1c78f67882f598b75b59dd4924b905e2`
- audiopipe: `780ef2d86f4186b34a7238d17ed5b7b7a7a31e0f`
- Orukeet: `oruk/orukeet@e31d65f0e6aaeec1cd4e84ed55e52ceef6a75717`, `onnx/combined-v0.1.0-int8`
- Baseline: `istupakov/parakeet-tdt-0.6b-v3-onnx@8f23f0c03c8761650bdb5b40aaf3e40d2c15f1ce`, int8 encoder and combined decoder/joiner

Local directory names in published logs and JSONL `source` fields are replaced with placeholders. Timings, counts, references and hypotheses are unchanged; equality was checked before publication. [Artifact hashes](artifact-hashes.json) record original and published bytes. Fixture audio remains in upstream Git LFS; no model weights or media are copied here.

## Established accuracy evidence

These previously measured results explain why Orukeet is useful as an option. They are separate from the new Screenpipe measurements below.

| Complete comparison | Recordings | Parakeet WER | Orukeet WER |
|---|---:|---:|---:|
| FLEURS, all 25 supported languages | 20,146 | 11.01% | 9.85% |
| Accent/domain sample, all 47 partitions | 12,006 | 16.72% | 15.25% |
| LibriSpeech test-clean | 2,620 | 1.53% | 1.46% |
| LibriSpeech test-other | 2,939 | 3.14% | 2.86% |

Orukeet wins 61/74 reported partitions, including 23/25 FLEURS languages. The English accent/domain subset wins all 20 partitions (9.51% → 8.84%, 5,120 recordings) and is already included in the table. These are prior NeMo FP32/BF16 greedy-batch measurements. LibriSpeech test-other was used for adaptation/selection, and 6,118 accent/domain recordings were included in preceding adaptation. They are not all untouched holdout estimates. [Complete pinned source, all losses, and measurement conditions](https://github.com/Oruk-AI/orukeet/blob/2269e42fb5b0857be1a78a4db3f2d7844591dcb5/docs/current-checkpoint-benchmarks.md).

The prior OpenWhispr int8 comparison covers all 640 selected clips across ten corpora: pooled WER **11.93% → 11.40%**, or 52 fewer errors (1,167 → 1,115 / 9,783 reference words). Seven corpora improve; three regress. Median latency was effectively tied. This used sherpa-onnx and a split-graph export, not this PR's audiopipe combined graph. [Complete application benchmark](https://github.com/Oruk-AI/orukeet/blob/2269e42fb5b0857be1a78a4db3f2d7844591dcb5/evidence/onnx-r3-20260910/APP_BENCHMARKS_TABLE.md).

## Fresh Screenpipe CPU ONNX comparison

Apple M5 Max, 128 GiB, 18 logical CPUs, macOS 26.4.1. Six existing `bench_quality` fixtures, 223.08 seconds of English audio, 491 normalized reference words. Both models use the same audiopipe runtime, one intra-op and one inter-op thread, mono 16 kHz PCM16 and the active session's 30-second chunk cap. Model order is Parakeet / Orukeet / Orukeet / Parakeet, three corpus passes per block, two excluded warm-ups per process: 36 timed calls/model.

| Metric | Parakeet v3 int8 | Orukeet int8 |
|---|---:|---:|
| Median processing time per fixture | 1,676.1 ms | 1,470.0 ms |
| p95 processing time per fixture | 3,118.6 ms | 2,759.0 ms |
| Audio seconds / processing second | 19.84× | 22.38× |
| Corpus WER | 20.37% (100/491) | 20.57% (101/491) |
| Mean fixture WER | 27.06% | 28.54% |
| Peak RSS, separate process run | 2,361,769,984 B | 2,619,293,696 B |
| Cached model load, single observation | 0.671 s | 0.728 s |

Median processing time was **12.3% lower**, with one extra word error overall and 10.9% higher peak RSS. Timing includes sample preprocessing, model inference and joining chunk text, but excludes download/load/warm-up/file I/O, capture, VAD, diarization and database writes. This compares CPU ONNX models, **not Screenpipe's default macOS MLX/GPU path**. It is not a time-to-first-word, battery or all-day memory measurement.

| Fixture | Reference words | Parakeet errors | Orukeet errors |
|---|---:|---:|---:|
| accuracy1 | 80 | 22 | 22 |
| accuracy2 | 158 | 27 | 26 |
| accuracy3 | 28 | 5 | 8 |
| accuracy4 | 82 | 23 | 22 |
| accuracy5 | 31 | 22 | 22 |
| poetic_kapil_gupta | 112 | 1 | 1 |

The actual `screenpipe-eval-transcription` binary reproduced the same six Orukeet hypotheses. The included legacy `bench_quality` diagnostic explores nine configurations with different resampling/normalization/overlap rules; its WER values are separate from this matched comparison. The fixture directory supplied to the eval uses its accepted FLAC layout; these are **Screenpipe fixtures, not LibriSpeech samples**.

## Reproduce

Requirements: Rust, Python 3, ffmpeg, Git LFS, both pinned model directories. `MODEL_DIR` must contain the int8 encoder, combined decoder/joiner and vocabulary. [Model hashes and run metadata](benchmark/meta.json), [fixture hashes and exact references](screenpipe-corpus.json), [raw output and summary](benchmark).

```sh
# In this evidence directory; SCREENPIPE is the integration checkout:
python3 prepare_corpus.py "$SCREENPIPE"

# In an audiopipe checkout at the pinned revision:
cp "$EVIDENCE/orukeet_compare.rs" examples/orukeet_compare.rs
cargo build --release --no-default-features --features parakeet,ort-defaults --example orukeet_compare

# In the evidence directory, with BENCH pointing to that release binary:
"$BENCH" "$PARAKEET_MODEL_DIR" screenpipe-corpus.json 3 > benchmark/0-parakeet.jsonl 2> benchmark/0-parakeet.log
"$BENCH" "$ORUKEET_MODEL_DIR" screenpipe-corpus.json 3 > benchmark/1-orukeet.jsonl 2> benchmark/1-orukeet.log
"$BENCH" "$ORUKEET_MODEL_DIR" screenpipe-corpus.json 3 > benchmark/2-orukeet.jsonl 2> benchmark/2-orukeet.log
"$BENCH" "$PARAKEET_MODEL_DIR" screenpipe-corpus.json 3 > benchmark/3-parakeet.jsonl 2> benchmark/3-parakeet.log
# macOS peak RSS, separate single-pass processes:
/usr/bin/time -l "$BENCH" "$PARAKEET_MODEL_DIR" screenpipe-corpus.json 1 > /dev/null 2> benchmark/memory-parakeet.log
/usr/bin/time -l "$BENCH" "$ORUKEET_MODEL_DIR" screenpipe-corpus.json 1 > /dev/null 2> benchmark/memory-orukeet.log
python3 score.py
```

`python3 score.py` can also re-aggregate the supplied results without loading models.

## Automated validation

Rust stable 1.97.1 and Bun 1.4.2. `HF_HOME` was an isolated cache staged with already-downloaded, hash-verified model files.

```sh
HF_HOME=/path/to/staged-hf-cache RUSTUP_TOOLCHAIN=stable cargo test -p screenpipe-audio --no-default-features --features parakeet --lib orukeet -- --include-ignored
# 3 passed, including the cached real-model/shared-session test
RUSTUP_TOOLCHAIN=stable cargo test -p screenpipe-audio --no-default-features --features parakeet --lib core::engine::tests
# 10 passed
RUSTUP_TOOLCHAIN=stable cargo test -p screenpipe-config --lib
# 43 passed
RUSTUP_TOOLCHAIN=stable cargo check -p screenpipe-engine --no-default-features --features parakeet,redact-onnx-cpu
# passed
RUSTUP_TOOLCHAIN=stable cargo clippy -p screenpipe-audio --no-default-features --features parakeet --lib --tests --no-deps
# passed, existing warnings
HF_HOME=/path/to/staged-hf-cache RUSTUP_TOOLCHAIN=stable cargo run -p screenpipe-audio --example bench_quality --no-default-features --features parakeet -- orukeet
# completed all 9 configurations
HF_HOME=/path/to/staged-hf-cache RUSTUP_TOOLCHAIN=stable cargo run -p screenpipe-audio-eval --bin screenpipe-eval-transcription -- --models orukeet --max-utterances 6 --librispeech-dir /path/to/repo-fixtures
# completed all 6 fixtures

# From apps/screenpipe-app-tauri:
bun x vitest run --config vitest.config.ts lib/__tests__/language-support.test.ts lib/hooks/managed-settings.test.ts
# 37 passed
bun run typecheck
# passed
```

The broader clippy command without `--no-deps` stops on the pre-existing `clippy::while_immutable_condition` error at `screenpipe-db/src/storage/in_place.rs:218`, unchanged from the base commit. [Validation logs](validation).

The browser-mock loop (`bun run dev:web`) loaded the settings route but could not validate the picker: upstream mock commands return null for onboarding/audio-device data, causing errors at `use-learning-window.ts:77` and `recording-settings.tsx:3611`. No browser/manual native test success or before/after recording is claimed. Native check status is recorded separately in [desktop validation](validation/desktop.md).

Implementation, measurements and the checks listed here were performed with Codex at agent autonomy. Human ownership/verification attestations remain for the contributor to complete.
