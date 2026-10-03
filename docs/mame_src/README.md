# MAME Galaxian driver sources (read-only reference)

Verbatim copies of the 5 files from the MAME `src/mame/galaxian/` driver, used as
reference material for KB extraction (T0.2.2) and MAME comparison (Phase 1+).
These are **not** compiled by this repo; treat them as read-only.

## Provenance

Synced byte-identical from MAME upstream `master`, branch of record:
`https://github.com/mamedev/mame/tree/master/src/mame/galaxian` (fetched 2026-10-02).
The original local copy at `H:/__Emulator__/Arcade_Galaxians - Copie/mame/galaxian/`
predated upstream master; on 2026-10-02 the directory was re-synced from upstream so
that KB extraction (T0.2.2) and MAME comparison (Phase 1+) reference current sources.

SHA256 of each file is recorded in `SHA256SUMS.txt` (generated with `sha256sum` from
inside this directory; validate with `sha256sum -c SHA256SUMS.txt`).

## Files

| File           | Lines  | Notes                                                        |
|----------------|--------|--------------------------------------------------------------|
| galaxian.cpp   | 17339  | driver + CPU/memory map (license:BSD-3-Clause)              |
| galaxian.h     | 944    | shared declarations                                          |
| galaxian_a.cpp | 777    | audio (discrete analog circuitry)                            |
| galaxian_a.h   | 77     | audio declarations                                           |
| galaxian_v.cpp | 1545   | video (tilemap, sprites, missiles/shells, starfield)        |

Copyright holders per the source header: Aaron Giles, Couriersud, Stephane Humbert, Robbbert.

## Caveat

The upstream commit is not pinned in this directory; files were fetched from
`master` on 2026-10-02 (see `SHA256SUMS.txt`). If an exact-version comparison
against a specific MAME release becomes important (Phase 1+), re-pin by recording
the upstream commit SHA alongside the hashes.
