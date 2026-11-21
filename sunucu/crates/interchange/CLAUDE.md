# interchange

## What it does
- File formats of import and export (T2.26): safe XML reader (`xml.rs`), `BeerXML` (`beerxml.rs`), `BeerJSON` (`beerjson.rs`), Brewfather JSON (`brewfather.rs`), BeerSmith `.bsmx` (`bsmx.rs`), escaped CSV (`csv.rs`) and a stored ZIP writer (`zip.rs`).
- The neutral model is in `model.rs` (internal SI units). `convert.rs` fills defaults and builds a model from the version JSON of the API.

## What it does not touch
- It has no I/O and no database. It does not calculate. Source values (OG, IBU, color) in a file are not trusted.

## Rules
- Do not allow a DOCTYPE or an entity in XML. Keep the size, depth and element limits (`xml::Limits`).
- A parser change needs a fixture in `tests/fixtures`. Write fixtures yourself. Exception: `tests/fixtures/bsmx/recipe.bsmx` is a third-party sample, used as a test fixture only (see its `README`).
- `.bsmx` uses `xml::Dialect::Html` (HTML entities from a fixed table, control characters dropped). Units and index tables are in `bsmx.rs`; misc lines are not imported.
- The Brewfather format follows assumptions listed in `brewfather.rs`; to be verified with a real export.
- Code is in English. Notes for Claude use ASD-STE100.
- Start the commit message with the task number.

## How to test
- In `sunucu/`, run: `cargo test -p interchange`.
