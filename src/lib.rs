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
//! # use reporting_api::BareReport;
//! # let payload = r#"[{"age":500,"type":"network-error","url":"https://example.com/about/","user_agent":"Mozilla/5.0","body":{"referrer":"https://example.com/","sampling_fraction":0.5,"server_ip":"203.0.113.75","protocol":"h2","method":"POST","status_code":200,"elapsed_time":45,"phase":"application","type":"ok"}}]"#;
//! let reports: Vec<BareReport> = serde_json::from_str(payload).unwrap();
//! ```
//!
//! That's it!  The elements of the vector will represent each of the reports in this upload batch.
//! Each one is a "bare" report, which means that we haven't tried to figure out what type of
//! report this is, or which Rust type corresponds with that report type.  Instead, the raw body of
//! the report is available (in the [`body`][] field) as a `serde_json` [`Value`][].
//!
//! If you know which particular kind of report you want to process, you can use the bare report's
//! [`parse`][] method to convert it into a "parsed" report.  For instance, if you know you only
//! care about [Network Error Logging][] reports:
//!
//! ```
//! # use reporting_api::BareReport;
//! # use reporting_api::Report;
//! # use reporting_api::NEL;
//! # let payload = r#"[{"age":500,"type":"network-error","url":"https://example.com/about/","user_agent":"Mozilla/5.0","body":{"referrer":"https://example.com/","sampling_fraction":0.5,"server_ip":"203.0.113.75","protocol":"h2","method":"POST","status_code":200,"elapsed_time":45,"phase":"application","type":"ok"}}]"#;
//! # let reports: Vec<BareReport> = serde_json::from_str(payload).unwrap();
//! // Ignore both kinds of failure, returning a Vec<Report<NEL>>.
//! let nel_reports = reports
//!     .into_iter()
//!     .filter_map(BareReport::parse::<NEL>)
//!     .filter_map(Result::ok)
//!     .collect::<Vec<Report<NEL>>>();
//! ```
//!
//! [`BareReport`]: struct.BareReport.html
//! [`body`]: struct.BareReport.html#structfield.body
//! [`Value`]: https://docs.rs/serde_json/*/serde_json/value/enum.Value.html
//! [`parse`]: struct.BareReport.html#method.parse
//!
//! Note that [`parse`][]'s return value is wrapped in _both_ [`Option`][] _and_ [`Result`][].  The
//! outer [`Option`][] tells you whether or not the report is of the expected type.  If it is, the
//! inner [`Result`][] tells you whether we were able to parse the reports `body` field according
//! to that type's expected schema.  In this example, we therefore need two `filter_map` calls to
//! strip away any mismatches and errors, leaving us with a vector of `Report<NEL>` instances.
//!
//! [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
//! [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
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
//! #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
//! pub struct Lint {
//!     pub source_file: String,
//!     pub line: u32,
//!     pub column: u32,
//!     pub finding: String,
//! }
//! ```
//!
//! Lastly, you must implement the [`ReportType`][] trait for your new type, which defines the
//! value of the `type` field in the report payload that corresponds to this new report type.
//!
//! [`ReportType`]: trait.ReportType.html
//!
//! ```
//! # use reporting_api::ReportType;
//! # pub struct Lint;
//! impl ReportType for Lint {
//!     fn report_type() -> &'static str {
//!         "lint"
//!     }
//! }
//! ```
//!
//! And that's it!  The [`parse`][] method will now work with your new report type.

use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// Represents a single report uploaded via the Reporting API, whose body is still a JSON object
/// and has not yet been parsed into a more specific Rust type.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BareReport {
    /// The amount of time between when the report was generated by the user agent and when it was
    /// uploaded.
    #[serde(with = "parse_milliseconds")]
    pub age: Duration,
    /// The URL of the request that this report describes.
    pub url: String,
    /// The value of the `User-Agent` header of the request that this report describes.
    pub user_agent: String,
    /// The type of report
    #[serde(rename = "type")]
    pub report_type: String,
    /// The body of the report, still encoded as a JSON object.
    pub body: Value,
}

impl BareReport {
    /// Verifies that a bare report has a particular type, and tries to parse the report body using
    /// the corresponding Rust type.  Returns `Some(Ok(...))` if everything goes well.  Returns
    /// `None` if the report has a different type, and `Some(Err(...))` if the report has the right
    /// type but we can't parse the report body using that type's schema.
    pub fn parse<C>(self) -> Option<Result<Report<C>, serde_json::Error>>
    where
        C: ReportType + for<'de> Deserialize<'de>,
    {
        if self.report_type != C::report_type() {
            return None;
        }
        Some(self.parse_body())
    }

    fn parse_body<C>(self) -> Result<Report<C>, serde_json::Error>
    where
        C: for<'de> Deserialize<'de>,
    {
        Ok(Report {
            age: self.age,
            url: self.url,
            user_agent: self.user_agent,
            body: serde_json::from_value(self.body)?,
        })
    }
}

/// Represents a single report, after having parsed the body into the Rust type specific to this
/// type of report.
#[derive(Clone, Debug, PartialEq)]
pub struct Report<C> {
    /// The amount of time between when the report was generated by the user agent and when it was
    /// uploaded.
    pub age: Duration,
    /// The URL of the request that this report describes.
    pub url: String,
    /// The value of the `User-Agent` header of the request that this report describes.
    pub user_agent: String,
    /// The body of the report.
    pub body: C,
}

/// A trait that maps each Rust report type to the corresponding `type` value that appears in a
/// JSON report payload.
pub trait ReportType {
    /// The value of the report's `type` field for reports of this type.
    fn report_type() -> &'static str;
}

/// The body of a single Network Error Logging report.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct NEL {
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

impl ReportType for NEL {
    fn report_type() -> &'static str {
        "network-error"
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
        Ok(Option::<u64>::deserialize(deserializer)?.map(Duration::from_millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn can_parse_unknown_report_type() {
        let report_json = json!({
            "age": 500,
            "url": "https://example.com/about/",
            "user_agent": "Mozilla/5.0",
            "type": "unknown",
            "body": {},
        });
        let report: BareReport =
            serde_json::from_value(report_json).expect("Should be able to parse JSON report");
        assert_eq!(
            report,
            BareReport {
                age: Duration::from_millis(500),
                url: "https://example.com/about/".to_string(),
                user_agent: "Mozilla/5.0".to_string(),
                report_type: "unknown".to_string(),
                body: json!({}),
            }
        );
    }

    #[test]
    fn cannot_parse_missing_report_type() {
        let report_json = json!({
            "age": 500,
            "url": "https://example.com/about/",
            "user_agent": "Mozilla/5.0",
            "body": {},
        });
        assert!(serde_json::from_value::<BareReport>(report_json).is_err());
    }

    #[test]
    fn cannot_parse_missing_body() {
        let report_json = json!({
            "age": 500,
            "url": "https://example.com/about/",
            "user_agent": "Mozilla/5.0",
            "type": "unknown",
        });
        assert!(serde_json::from_value::<BareReport>(report_json).is_err());
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
        let bare_report: BareReport =
            serde_json::from_value(report_json).expect("Should be able to parse JSON report");
        let report: Report<NEL> = bare_report
            .parse()
            .expect("Report should be a NEL report")
            .expect("Should be able to parse NEL report body");
        assert_eq!(
            report,
            Report {
                age: Duration::from_millis(500),
                url: "https://example.com/about/".to_string(),
                user_agent: "Mozilla/5.0".to_string(),
                body: NEL {
                    referrer: "https://example.com/".to_string(),
                    sampling_fraction: 0.5,
                    server_ip: "203.0.113.75".to_string(),
                    protocol: "h2".to_string(),
                    method: "POST".to_string(),
                    status_code: Some(200),
                    elapsed_time: Some(Duration::from_millis(45)),
                    phase: "application".to_string(),
                    status: "ok".to_string(),
                },
            }
        );
    }
}
