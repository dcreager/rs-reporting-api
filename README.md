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
rs-reporting-api = "^0.2"
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
let reports: Vec<Report> = serde_json::from_str(payload).unwrap();
```

That's it!  The elements of the vector will represent each of the reports in
this upload batch.  Each [`Report`][] instance will contain the fields defined
by the [Reporting API][] for all report types, and also a [`body`][] containing
the fields specific to that type of report.  You can use the [`body`][]'s
[`is`][] and [`downcast_ref`][] methods if you know which particular kind of
report you want to process.  For instance, if you know you only care about
[Network Error Logging][] reports:

``` rust
// Will be an Iterator<Item = &NELReport>
let nel_content = reports.iter().filter_map(|report| report.body.downcast_ref::<NELReport>());
```

[`Report`]: struct.Report.html
[`body`]: struct.Report.html#structfield.body
[`is`]: struct.ReportBody.html#method.is
[`downcast_ref`]: struct.ReportBody.html#method.downcast_ref

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
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct LintReport {
    pub source_file: String,
    pub line: u32,
    pub column: u32,
    pub finding: String,
}
```

Note that you need to derive a couple of traits, and your type must be `'static`
(i.e., it cannot contain any references).

Lastly, you must implement the [`ReportPayload`][] trait for your new type.
Every impl of this trait looks exactly the same; you can copy-paste the
following, replacing only the type name (`LintReport`) and the value of the
`name` parameter in the annotation (`lint`).  (This is the value of the `type`
field in the report payload that corresponds to this new report type.)

[`ReportPayload`]: trait.ReportPayload.html

``` rust
#[typetag::serde(name = "lint")]
impl ReportPayload for LintReport {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_debug(&self) -> &dyn Debug {
        self
    }

    fn eq_payload(&self, other: &dyn ReportPayload) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .map_or(false, |other| self == other)
    }
}
```

And that's it!  The `typetag::serde` annotation is the magic that ties
everything together; it automatically causes our deserialization logic to look
for this type name when deserializing, and delegate to your new type to
deserialize the contents of the `body` field.
