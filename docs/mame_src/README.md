# MAME Galaxian driver sources (read-only reference)

Verbatim copies of the 5 files from the MAME `src/mame/galaxian/` driver, used as
reference material for KB extraction (T0.2.2) and MAME comparison (Phase 1+).
These are **not** compiled by this repo; treat them as read-only.

## Provenance

Copied byte-identical from the local copy at:

    H:/__Emulator__/Arcade_Galaxians - Copie/mame/galaxian/

SHA256 of each file was verified before and after the copy (see `SHA256SUMS.txt`,
generated with `sha256sum` from inside this directory; validate with
`sha256sum -c SHA256SUMS.txt`).

## Files

| File           | Lines  | Notes                                                        |
|----------------|--------|--------------------------------------------------------------|
| galaxian.cpp   | 17229  | driver + CPU/memory map (license:BSD-3-Clause)              |
| galaxian.h     | 948    | shared declarations                                          |
| galaxian_a.cpp | 777    | audio (discrete analog circuitry)                            |
| galaxian_a.h   | 77     | audio declarations                                           |
| galaxian_v.cpp | 1545   | video (tilemap, sprites, missiles/shells, starfield)        |

Copyright holders per the source header: Aaron Giles, Couriersud, Stephane Humbert, Robbbert.

## Caveat

The exact MAME version of this local copy is **not pinned** anywhere in it; the
files were taken as-is from the directory above without a version stamp. If an
exact-version comparison against upstream MAME becomes important (Phase 1+),
re-pin by identifying which MAME release these files match.
