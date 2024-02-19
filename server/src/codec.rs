use bytes::BytesMut;
use protocol::common::{nom_error_to_anyhow, Parse};

#[derive(Default)]
pub struct MinigolfCodec {
    received_buf: BytesMut,
}

impl MinigolfCodec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn accept(&mut self, bytes: &[u8]) {
        self.received_buf.extend(bytes);
    }
    pub fn next_packet<T>(&mut self) -> anyhow::Result<Option<T>>
    where
        T: Parse,
    {
        if self.received_buf.is_empty() {
            return Ok(None);
        }
        let packet_string = String::from_utf8_lossy(&self.received_buf);

        let (input, parse) = <T>::parse(&packet_string).map_err(|e| nom_error_to_anyhow(e))?;
        self.received_buf = input.into();

        Ok(Some(parse))
    }
}
