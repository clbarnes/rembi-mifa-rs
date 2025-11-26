# rembi-mifa

Data models for FAIR bioimaging data, REMBI [^1] and MIFA [^2].

This crates uses [`jiff`](https://docs.rs/jiff) for dates/times
(specifically [`civil::Date`](https://docs.rs/jiff/latest/jiff/civil/struct.Date.html) for dates
and [`Zoned`](https://docs.rs/jiff/latest/jiff/struct.Zoned.html) for datetimes),
[`url`](https://docs.rs/url) for URLs, and [`iref`](https://docs.rs/iref) for URIs.

[^1]: <https://www.ebi.ac.uk/bioimage-archive/rembi-model-reference/>
[^2]: <https://www.ebi.ac.uk/bioimage-archive/mifa-model-reference/>

## Notes

Most types provide a `new` method which populates the minimum required fields.
Further fields can be set with direct field access.

### Validation

It is possible for the types to contain invalid data.
Use the [Valid] wrapper for serialising and deserialising to ensure that no invalid data is ingested or written.

```rust
use rembi_mifa::{rembi::RembiStudy, Valid};
let maybe_invalid = RembiStudy::new(...);
let valid = Valid::try_new(maybe_invalid).expect("should be valid");

// serialises transparently
let valid_str = serde_json::to_string(valid);

// would refuse to deserialise if the data were invalid
let valid_deser: Valid<RembiStudy> = serde_json::from_str(&valid_str);

// extract the data, or use `.inner()` to get a reference
let deser = valid_deser.into_inner();
```

### Name collisions

Where possible, this crate uses the exact type/ field names from the specifications.
This means that there are some types with the same name but different fields.

We recommended that, in contexts where the specification in use is unclear,
the types are always referred to in namespaced form i.e.

```rust
use rembi_mifa::{rembi, mifa};
let a1 = rembi::Author::new(...);
let a2 = mifa::Author::new(...);
```

### Specification ambiguity

In some cases, the specification is missing definitions or otherwise unclear.

Particular notes are

- [rembi::License], defined as empty; here we use an empty struct
- [rembi::AnnotationType], referred to but not defined; here we re-export [mifa::AnnotationType]
- some list ([Vec]) or FreeText/str (String) fields are required, but not strictly required to be non-empty. This crate
  - always distinguishes between omitted and empty strings
  - always serialises required list fields, even if they're empty
  - omits optional list fields if they're empty
    - there are a few exceptions where it is helpful to distinguish between "not specified" and "specifically zero items"

### Strictness

This crate tries to be somewhat permissive about what it can deserialise,
but is opinionated on how certain fields are serialised.
This includes

- ORCiD IDs are serialised as HTTPS URLs [as recommended by ORCiD](https://support.orcid.org/hc/en-us/articles/360006897674-Structure-of-the-ORCID-Identifier)
  - they can be parsed from HTTP URLs, or just the last path segment, and/or with or without hyphens
- DOIs are serialised in minimal form like `10.1234/deadbeef` [as recommended in the DOI Handbook](https://www.doi.org/doi-handbook/HTML/index.html) where "the context clearly indicates that a DOI name is implied"
  - they can be parsed from any URI; most commonly HTTP(s) URLs or URIs with scheme `doi:`
  - note that organisations such as APA and DataCite recommend presenting DOIs as URLs under a particular proxy
- dates/ times are serialised/ deserialised according to [jiff::temporal]
