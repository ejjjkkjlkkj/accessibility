# Accessibility resource database

This directory contains generated catalog outputs. The canonical input is `data/resources.csv`; the generator is `tools/build_resource_db.py`.

## Outputs

- `accessibility_resources.sqlite`: SQLite database with `resources` and `metadata` tables.
- `resource_status.json`: machine-readable snapshot, counts, per-resource checks, and warning.
- `resource_status.csv`: spreadsheet-friendly snapshot.

## Resource fields

The source catalog records a stable ID, display name, domain, canonical URL, category, layer, provisional official-status label, and notes. Generated rows add HTTP status, final redirect URL, reachability result, whether the submitted URL uses HTTPS, error type, and UTC check timestamp.

## Interpretation rules

- `official-candidate` is a research label, not verified proof of ownership or affiliation.
- `reachable=true` means the HTTP check received a successful response; it does not prove content correctness, trustworthiness, accessibility conformance, or official status.
- `reachable=false` means the checker could not verify reachability during this run. Rate limits, bot protection, HEAD restrictions, temporary outages, and network errors can cause false negatives.
- `tls=true` only records that the submitted URL uses HTTPS. Certificate validation is performed by Python's default TLS context, but this does not establish that a site is legitimate.
- A redirected URL is recorded for review and must not silently replace the catalog's canonical URL.

## Refresh and validation

GitHub Actions runs offline unit tests before the network scan. The scheduled workflow runs each Monday at 05:23 UTC; it can also be started manually from the repository's Actions tab. The workflow uploads a 90-day artifact and commits changed generated outputs to `main`.

The catalog is a seed, not a complete worldwide directory. Each resource still needs source evidence, accessibility relevance, official-affiliation verification, and periodic human review. No firmware, NVRAM, security settings, or physical device state is modified.
