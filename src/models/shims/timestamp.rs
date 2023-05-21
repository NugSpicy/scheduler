use chrono::{Duration, NaiveDateTime};
use derive_more::From;
use scylla::_macro_internal::CqlValue;
use scylla::cql_to_rust::FromCqlVal;
use scylla::frame::value::Timestamp as ScyllaTs;

use serde::Serialize;

#[derive(From, Debug)]
pub struct Timestamp(ScyllaTs);

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let datetime = NaiveDateTime::from_timestamp_millis(self.0 .0.num_milliseconds()).unwrap();
        serializer.serialize_str(&datetime.to_string())
    }
}

impl scylla::frame::value::Value for Timestamp {
    fn serialize(
        &self,
        buf: &mut Vec<u8>,
    ) -> std::result::Result<(), scylla::frame::value::ValueTooBig> {
        self.0.serialize(buf)
    }
}

impl FromCqlVal<CqlValue> for Timestamp {
    fn from_cql(cql_val: CqlValue) -> Result<Self, scylla::cql_to_rust::FromCqlValError> {
        ScyllaTs::from_cql(cql_val).map(|res| res.into())
    }
}

impl From<Duration> for Timestamp {
    fn from(duration: Duration) -> Self {
        Timestamp(ScyllaTs(duration))
    }
}
