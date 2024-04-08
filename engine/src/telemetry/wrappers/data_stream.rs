use std::pin::Pin;

use futures_util::Stream;
use models::event::DataEvent;

pub fn instrument_stream(
    data_event: Pin<Box<dyn Stream<Item = eyre::Result<DataEvent>> + Send>>,
) -> Pin<Box<dyn Stream<Item = eyre::Result<DataEvent>> + Send>> {
    // TODO: record data event counter
    // TODO: record data event error count
    todo!()
}
