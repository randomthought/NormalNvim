use std::pin::Pin;

use domain::event::model::DataEvent;
use futures_util::Stream;

pub fn instrument_stream(
    data_event: Pin<Box<dyn Stream<Item = eyre::Result<DataEvent>> + Send>>,
) -> Pin<Box<dyn Stream<Item = eyre::Result<DataEvent>> + Send>> {
    // TODO: record data event counter
    // TODO: record data event error count
    todo!()
}
