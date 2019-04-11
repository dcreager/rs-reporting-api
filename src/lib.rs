// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2019, rs-reporting-api authors.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License.  You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied.  See the License for the specific language governing permissions and
// limitations under the License.
// ------------------------------------------------------------------------------------------------

//! This crate provides some useful Rust code for working with the [Reporting API][] and [Network
//! Error Logging][] W3C draft specifications.
//!
//! [Reporting API]: https://w3c.github.io/reporting/
//! [Network Error Logging]: https://w3c.github.io/network-error-logging/
//!
//! # Overview
//!
//! The core of the [Reporting API][] is pretty simple: reports are uploaded via a `POST` to a URL
//! of your choosing.  The payload of the `POST` request is a JSON-encoded array of reports, and
//! the report schema is defined by the spec.
//!
//! The [Reporting API][] can be used to upload many different _kinds_ of reports.  For instance,
//! Reporting itself defines [crash reports][], [deprecations][], and [interventions][], all of
//! which come from the JavaScript environment running in the browser.  Other report types are
//! complex enough that they need to be defined in their own specs, such as [Network Error
//! Logging][] and [Content Security Policy][].  Regardless of where they're defined, each report
//! type defines some fields specific to that type (the **_body_**), and the [Reporting API][]
//! defines some fields that are common to all types.
//!
//! [crash reports]: https://w3c.github.io/reporting/#crash-report
//! [deprecations]: https://w3c.github.io/reporting/#deprecation-report
//! [interventions]: https://w3c.github.io/reporting/#intervention-report
//! [Content Security Policy]: https://www.w3.org/TR/CSP3/
//!
//! This library provides a definition of all of these schemas as regular Rust types, along with
//! the ability to use [serde][] to serialize and deserialize them.  We've carefully defined
//! everything so that [serde_json][] will automatically do The Right Thing and use a JSON
//! serialization that lines up with the various specifications.  We also provide way to define
//! body schemas for new report types, and have them seamlessly fit in with the rest of the
//! serialization logic.
//!
//! [serde]: https://docs.rs/serde/
//! [serde_json]: https://docs.rs/serde_json/
//!
//! # Collecting reports
//!
//! The simplest way to use this library is if you just want to receive reports from somewhere
//! (you're implementing a collector, for instance, and we've already defined Rust types for all of
//! the report types that you care about).
//!
//! To do that, you just need to use `serde_json` to deserialize the content of the JSON string
//! that you've received:
//!
//! ```
//! # use reporting_api::Report;
//! # let payload = r#"[{"age":500,"type":"network-error","url":"https://example.com/about/","user_agent":"Mozilla/5.0","body":{"referrer":"https://example.com/","sampling_fraction":0.5,"server_ip":"203.0.113.75","protocol":"h2","method":"POST","status_code":200,"elapsed_time":45,"phase":"application","type":"ok"}}]"#;
//! let reports: Vec<Report> = serde_json::from_str(payload).unwrap();
//! ```
//!
//! That's it!  The elements of the vector will represent each of the reports in this upload batch.
//! Each [`Report`][] instance will contain the fields defined by the [Reporting API][] for all
//! report types, and also a [`body`][] containing the fields specific to that type of report.  You
//! can use the [`body`][]'s [`is`][] and [`downcast_ref`][] methods if you know which particular
//! kind of report you want to process.  For instance, if you know you only care about [Network
//! Error Logging][] reports:
//!
//! ```
//! # use reporting_api::Report;
//! # use reporting_api::NELReport;
//! # let payload = r#"[{"age":500,"type":"network-error","url":"https://example.com/about/","user_agent":"Mozilla/5.0","body":{"referrer":"https://example.com/","sampling_fraction":0.5,"server_ip":"203.0.113.75","protocol":"h2","method":"POST","status_code":200,"elapsed_time":45,"phase":"application","type":"ok"}}]"#;
//! # let reports: Vec<Report> = serde_json::from_str(payload).unwrap();
//! // Will be an Iterator<Item = &NELReport>
//! let nel_content = reports.iter().filter_map(|report| report.body.downcast_ref::<NELReport>());
//! ```
//!
//! [`Report`]: struct.Report.html
//! [`body`]: struct.Report.html#structfield.body
//! [`is`]: struct.ReportBody.html#method.is
//! [`downcast_ref`]: struct.ReportBody.html#method.downcast_ref
//!
//! # Creating a new report type
//!
//! This should be a relatively rare occurrence, but consider a new report type that uses the
//! [Reporting API][] but that isn't covered here.  For instance, let's say there's a new `lint`
//! report type whose body content looks like:
//!
//! ``` json
//! {
//!     "source_file": "foo.js",
//!     "line": 10,
//!     "column": 12,
//!     "finding": "Indentation doesn't match the rest of the file"
//! }
//! ```
//!
//! First you'll define a Rust type to hold the body content:
//!
//! ```
//! # use serde::Deserialize;
//! # use serde::Serialize;
//! #[derive(Debug, Deserialize, PartialEq, Serialize)]
//! pub struct LintReport {
//!     pub source_file: String,
//!     pub line: u32,
//!     pub column: u32,
//!     pub finding: String,
//! }
//! ```
//!
//! Note that you need to derive a couple of traits, and your type must be `'static` (i.e., it
//! cannot contain any references).
//!
//! Lastly, you must implement the [`ReportPayload`][] trait for your new type.  Every impl of this
//! trait looks exactly the same; you can copy-paste the following, replacing only the type name
//! (`LintReport`) and the value of the `name` parameter in the annotation (`lint`).  (This is the
//! value of the `type` field in the report payload that corresponds to this new report type.)
//!
//! [`ReportPayload`]: trait.ReportPayload.html
//!
//! ```
//! # use std::any::Any;
//! # use std::fmt::Debug;
//! # use reporting_api::ReportPayload;
//! # use serde::Deserialize;
//! # use serde::Serialize;
//! # #[derive(Debug, Deserialize, PartialEq, Serialize)]
//! # pub struct LintReport;
//! #[typetag::serde(name = "lint")]
//! impl ReportPayload for LintReport {
//!     fn as_any(&self) -> &dyn Any {
//!         self
//!     }
//!
//!     fn as_debug(&self) -> &dyn Debug {
//!         self
//!     }
//!
//!     fn eq_payload(&self, other: &dyn ReportPayload) -> bool {
//!         other
//!             .as_any()
//!             .downcast_ref::<Self>()
//!             .map_or(false, |other| self == other)
//!     }
//! }
//! ```
//!
//! And that's it!  The `typetag::serde` annotation is the magic that ties everything together; it
//! automatically causes our deserialization logic to look for this type name when deserializing,
//! and delegate to your new type to deserialize the contents of the `body` field.

use std::any::Any;
use std::fmt::Debug;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

/// Represents a single report uploaded via the Reporting API.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Report {
    /// The amount of time between when the report was generated by the user agent and when it was
    /// uploaded.
    #[serde(with = "parse_milliseconds")]
    pub age: Duration,
    /// The URL of the request that this report describes.
    pub url: String,
    /// The value of the `User-Agent` header of the request that this report describes.
    pub user_agent: String,
    /// The body of the report.
    #[serde(flatten)]
    pub body: ReportBody,
}

/// Contains the body of a single report.  The actual content for each kind of report is stored in
/// its own Rust type, which must implement the [`ReportPayload`][] trait.
///
/// [`ReportPayload`]: trait.ReportPayload.html
#[derive(Deserialize, Serialize)]
pub struct ReportBody(Box<dyn ReportPayload>);

impl ReportBody {
    /// Create a new report body containing the given payload content.
    pub fn new<P: ReportPayload>(payload: P) -> ReportBody {
        ReportBody(Box::new(payload))
    }

    /// Returns whether the content of this report body has a given type.
    pub fn is<P: ReportPayload>(&self) -> bool {
        self.0.as_any().is::<P>()
    }

    /// Returns a reference to the body content if it's of type `P`, or `None` if it isn't.
    pub fn downcast_ref<P: ReportPayload>(&self) -> Option<&P> {
        self.0.as_any().downcast_ref::<P>()
    }
}

impl Debug for ReportBody {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.as_debug().fmt(f)
    }
}

impl PartialEq for ReportBody {
    fn eq(&self, other: &ReportBody) -> bool {
        self.0.eq_payload(other.0.as_ref())
    }
}

/// Each kind of report that can be delivered via the Reporting API will have its own type, which
/// implements this trait.  Each type that implements this trait must also implement
/// [`std::any::Any`][Any], [`std::fmt::Debug`][Debug], and [`std::cmp::PartialEq`][PartialEq].
///
/// [Any]: https://doc.rust-lang.org/std/any/trait.Any.html
/// [Debug]: https://doc.rust-lang.org/std/fmt/trait.Debug.html
/// [PartialEq]: https://doc.rust-lang.org/std/cmp/trait.PartialEq.html
#[typetag::serde(tag = "type", content = "body")]
pub trait ReportPayload: 'static {
    /// Returns a reference to this payload as a `std::any::Any`.
    fn as_any(&self) -> &dyn Any;
    /// Returns a reference to this payload as a `std::fmt::Debug`.
    fn as_debug(&self) -> &dyn Debug;
    /// Compares this payload with another of an arbitrary type, returning `false` if the two
    /// payloads have different types.
    fn eq_payload(&self, other: &dyn ReportPayload) -> bool;
}

/// A single Network Error Logging report.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct NELReport {
    /// The referrer information for the request, as determined by the referrer policy associated
    /// with its client.
    pub referrer: String,
    /// The sampling rate that was in effect for this request, expressed as a frcation between 0.0
    /// and 1.0 (inclusive).
    pub sampling_fraction: f32,
    /// The IP address of the host to which the user agent sent the request.
    pub server_ip: String,
    /// The ALPN ID of the network protocol used to fetch the resource.
    pub protocol: String,
    /// The method of the HTTP request (e.g., `GET`, `POST`)
    pub method: String,
    /// The status code of the HTTP response, if available.
    pub status_code: Option<u16>,
    /// The elapsed time between the start of the resource fetch and when it was completed or
    /// aborted by the user agent.
    #[serde(with = "parse_opt_milliseconds")]
    pub elapsed_time: Option<Duration>,
    /// The phase of the request in which the failure occurred, if any.  One of `dns`,
    /// `connection`, or `application`.  A successful request always has a phase of `application`.
    pub phase: String,
    /// The code describing the error that occurred, or `ok` if the request was successful.  See
    /// the NEL spec for the [authoritative
    /// list](https://w3c.github.io/network-error-logging/#predefined-network-error-types) of
    /// possible codes.
    #[serde(rename = "type")]
    pub status: String,
}

#[typetag::serde(name = "network-error")]
impl ReportPayload for NELReport {
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

/// A serde parsing module that can be used to parse durations expressed as an integer number of
/// milliseconds.
pub mod parse_milliseconds {
    use std::time::Duration;

    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;

    pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(value.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Duration::from_millis(u64::deserialize(deserializer)?))
    }
}

/// A serde parsing module that can be used to parse _optional_ durations expressed as an integer
/// number of milliseconds.
pub mod parse_opt_milliseconds {
    use std::time::Duration;

    use serde::Deserialize;
    use serde::Deserializer;
    use serde::Serializer;

    pub fn serialize<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(duration) => serializer.serialize_some(&(duration.as_millis() as u64)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Option::<u64>::deserialize(deserializer)?.map(|millis| Duration::from_millis(millis)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn cannot_parse_unknown_report_type() {
        let report_json = json!({
            "age": 500,
            "type": "unknown",
            "url": "https://example.com/about/",
            "user_agent": "Mozilla/5.0",
            "body": {},
        });
        assert!(serde_json::from_value::<Report>(report_json).is_err());
    }

    #[test]
    fn cannot_parse_missing_report_type() {
        let report_json = json!({
            "age": 500,
            "url": "https://example.com/about/",
            "user_agent": "Mozilla/5.0",
            "body": {},
        });
        assert!(serde_json::from_value::<Report>(report_json).is_err());
    }

    #[test]
    fn can_parse_nel_report() {
        let report_json = json!({
            "age": 500,
            "type": "network-error",
            "url": "https://example.com/about/",
            "user_agent": "Mozilla/5.0",
            "body": {
                "referrer": "https://example.com/",
                "sampling_fraction": 0.5,
                "server_ip": "203.0.113.75",
                "protocol": "h2",
                "method": "POST",
                "status_code": 200,
                "elapsed_time": 45,
                "phase":"application",
                "type": "ok"
            }
        });
        let report: Report =
            serde_json::from_value(report_json).expect("Should be able to parse JSON report");
        assert_eq!(
            report,
            Report {
                age: Duration::from_millis(500),
                url: "https://example.com/about/".to_string(),
                user_agent: "Mozilla/5.0".to_string(),
                body: ReportBody::new(NELReport {
                    referrer: "https://example.com/".to_string(),
                    sampling_fraction: 0.5,
                    server_ip: "203.0.113.75".to_string(),
                    protocol: "h2".to_string(),
                    method: "POST".to_string(),
                    status_code: Some(200),
                    elapsed_time: Some(Duration::from_millis(45)),
                    phase: "application".to_string(),
                    status: "ok".to_string(),
                }),
            }
        );
    }
}
