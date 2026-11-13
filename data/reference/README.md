# Country reference snapshot

`countries-iso-codes-4.20.1-1.json` is the pinned ISO 3166-1 country and territory snapshot. It contains 249 alpha-2/alpha-3 records with display and optional official names. The snapshot comes from Debian's `iso-codes` 4.20.1-1 package, generated on 2026-11-13. Its presence is reference data only; it does not indicate legal or operational availability.

The Rust API validates the snapshot's source, version, record count, code shape, and uniqueness, then installs it transactionally during database startup. The API validates account and preference country foreign keys after import and before it begins serving requests. The versioned JSON file is the single checked-in data source.

The source project is [iso-codes](https://salsa.debian.org/iso-codes-team/iso-codes); the reference standard is [ISO 3166](https://www.iso.org/iso-3166-country-codes.html). The dataset is distributed under LGPL-2.1-or-later. Copyright notices include Alastair McKinstry, Christian Perrier, and Dr. Tobias Quathamer; see upstream notices when refreshing the snapshot.

To generate a new versioned snapshot from an upstream ISO catalogue:

```bash
scripts/refresh_country_directory.sh /path/to/iso_3166-1.json 4.20.1-1
```

Review the source package/version, record count, and changes before updating the Rust snapshot version and reference-data documentation. The generation script requires `jq`; the backend runtime does not.
