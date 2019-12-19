//! Time series database which uses B+ trees to store tha data.

use super::handle::{make_handle, TsDbHandle};
use super::query::{Query, QueryResult};
use super::sample::{Sample, SampleMetrics};
use super::trace::Trace;
use super::{Aggregation, Observation};
use crate::time::TimeSpan;
use std::collections::HashMap;

/// A time series database which can be used as a library.
/// Note that this struct is not usable in multiple threads.
/// To make it accessible from multiple threads, use the TsDbHandle wrapper.
#[derive(Debug)]
pub struct TsDb {
    path: String,
    pub data: HashMap<String, Trace>,
}

impl std::fmt::Display for TsDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TsDb {} with {} traces", self.path, self.data.len())
    }
}

impl Default for TsDb {
    fn default() -> Self {
        let path = "x".to_string();
        let data = HashMap::new();
        Self { path, data }
    }
}

/// Database for time series.
impl TsDb {
    pub fn into_handle(self) -> TsDbHandle {
        make_handle(self)
    }

    pub fn get_signal_names(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    fn get_or_create_trace(&mut self, name: &str) -> &mut Trace {
        if !self.data.contains_key(name) {
            self.new_trace(name);
        }
        self.data.get_mut(name).unwrap()
    }

    /// Add a batch of values
    pub fn add_values(&mut self, name: &str, samples: Vec<Observation<Sample>>) {
        let trace = self.get_or_create_trace(name);
        trace.add_values(samples);
    }

    pub fn new_trace(&mut self, name: &str) {
        let trace = Trace::default();
        self.data.insert(name.to_string(), trace);
        // self.get_trace(name)
    }

    pub fn add_value(&mut self, name: &str, observation: Observation<Sample>) {
        let trace = self.get_or_create_trace(name);
        trace.add_sample(observation);
    }

    /// Query the given trace for data.
    pub fn query(&self, name: &str, query: Query) -> QueryResult {
        if let Some(trace) = self.data.get(name) {
            trace.query(query)
        } else {
            QueryResult { query, inner: None }
        }
    }

    /// Get a summary for a certain timerange (or all time) the given trace.
    pub fn summary(
        &self,
        name: &str,
        timespan: Option<&TimeSpan>,
    ) -> Option<Aggregation<Sample, SampleMetrics>> {
        self.data.get(name)?.summary(timespan)
    }
}
