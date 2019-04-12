# Reporting API and Network Error Logging

[![Build Status](https://api.travis-ci.org/dcreager/rs-reporting-api.svg?branch=master)](https://travis-ci.org/dcreager/rs-reporting-api)
[![Latest Version](https://img.shields.io/crates/v/reporting-api.svg)](https://crates.io/crates/reporting-api)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/reporting-api)

This crate provides some useful Rust code for working with the [Reporting API][]
and [Network Error Logging][] W3C draft specifications.

[Reporting API]: https://w3c.github.io/reporting/
[Network Error Logging]: https://w3c.github.io/network-error-logging/

``` toml
[dependencies]
reporting-api = "^0.2"
```

# Overview

The core of the [Reporting API][] is pretty simple: reports are uploaded via a
`POST` to a URL of your choosing.  The payload of the `POST` request is a
JSON-encoded array of reports, and the report schema is defined by the spec.

The [Reporting API][] can be used to upload many different _kinds_ of reports.
For instance, Reporting itself defines [crash reports][], [deprecations][], and
[interventions][], all of which come from the JavaScript environment running in
the browser.  Other report types are complex enough that they need to be defined
in their own specs, such as [Network Error Logging][] and [Content Security
Policy][].  Regardless of where they're defined, each report type defines some
fields specific to that type (the **_body_**), and the [Reporting API][] defines
some fields that are common to all types.

[crash reports]: https://w3c.github.io/reporting/#crash-report
[deprecations]: https://w3c.github.io/reporting/#deprecation-report
[interventions]: https://w3c.github.io/reporting/#intervention-report
[Content Security Policy]: https://www.w3.org/TR/CSP3/

This library provides a definition of all of these schemas as regular Rust
types, along with the ability to use [serde][] to serialize and deserialize
them.  We've carefully defined everything so that [serde_json][] will
automatically do The Right Thing and use a JSON serialization that lines up with
the various specifications.  We also provide way to define body schemas for new
report types, and have them seamlessly fit in with the rest of the serialization
logic.

[serde]: https://docs.rs/serde/
[serde_json]: https://docs.rs/serde_json/

# Collecting reports

The simplest way to use this library is if you just want to receive reports from
somewhere (you're implementing a collector, for instance, and we've already
defined Rust types for all of the report types that you care about).

To do that, you just need to use `serde_json` to deserialize the content of the
JSON string that you've received:

``` rust
let reports: Vec<BareReport> = serde_json::from_str(payload).unwrap();
```

That's it!  The elements of the vector will represent each of the reports in
this upload batch.  Each one is a "bare" report, which means that we haven't
tried to figure out what type of report this is, or which Rust type corresponds
with that report type.  Instead, the raw body of the report is available (in the
`body` field) as a `serde_json` [`Value`][].

If you know which particular kind of report you want to process, you can use the
bare report's `parse` method to convert it into a "parsed" report.  For
instance, if you know you only care about [Network Error Logging][] reports:

``` rust
// Ignore both kinds of failure, returning a Vec<Report<NEL>>.
let nel_reports = reports
    .into_iter()
    .filter_map(BareReport::parse::<NEL>)
    .filter_map(Result::ok)
    .collect::<Vec<Report<NEL>>>();
```

[`Value`]: https://docs.rs/serde_json/*/serde_json/value/enum.Value.html

Note that `parse`'s return value is wrapped in _both_ [`Option`][] _and_
[`Result`][].  The outer [`Option`][] tells you whether or not the report is of
the expected type.  If it is, the inner [`Result`][] tells you whether we were
able to parse the reports `body` field according to that type's expected schema.
In this example, we therefore need two `filter_map` calls to strip away any
mismatches and errors, leaving us with a vector of `Report<NEL>` instances.

[`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
[`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html

# Creating a new report type

This should be a relatively rare occurrence, but consider a new report type that
uses the [Reporting API][] but that isn't covered here.  For instance, let's say
there's a new `lint` report type whose body content looks like:

``` json
{
    "source_file": "foo.js",
    "line": 10,
    "column": 12,
    "finding": "Indentation doesn't match the rest of the file"
}
```

First you'll define a Rust type to hold the body content:

``` rust
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Lint {
    pub source_file: String,
    pub line: u32,
    pub column: u32,
    pub finding: String,
}
```

Lastly, you must implement the `ReportType` trait for your new type, which
defines the value of the `type` field in the report payload that corresponds to
this new report type.

``` rust
impl ReportType for Lint {
    fn report_type() -> &'static str {
        "lint"
    }
}
```

And that's it!  The `parse` method will now work with your new report type.
