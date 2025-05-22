//! Packet module.

use derive_more::derive::From;

use super::{
    header::Header,
    payload::{Payload, PayloadFromSliceError},
    payload_content::PayloadContent,
    raw_packet::{RawPacket, RawPacketFromSliceError, RawPacketWriteError},
};

/// A packet that contains some payload.
#[derive(Debug, Clone, PartialEq)]
pub struct Packet<'a> {
    /// The payload of the packet.
    pub payload: Payload<'a>,
}

impl<'a> Packet<'a> {
    /// Creates a new packet with the given payload.
    pub const fn new_raw(payload: Payload<'a>) -> Self {
        Self { payload }
    }

    /// Creates a new packet with the given payload.
    pub fn new(content: impl Into<PayloadContent<'a>>) -> Self {
        Self {
            payload: Payload::new(content),
        }
    }

    /// Returns a reference to the payload.
    pub const fn payload(&self) -> &Payload<'a> {
        &self.payload
    }

    /// Writes the packet to the given destination buffer.
    pub fn write_to(&self, dst: &mut [u8]) -> Result<usize, PacketWriteError> {
        Ok(RawPacket::write_to(&self.payload, dst)?)
    }

    /// Returns a reference to a packet if the given slice starts with a valid packet.
    pub fn maybe_packet_from_prefix(
        src: &'a mut [u8],
    ) -> Result<Option<(Packet<'a>, usize)>, PacketFromSliceError> {
        match RawPacket::maybe_raw_packet_from_prefix(src) {
            Err(err) => Err(PacketFromSliceError::RawPacket(err)),
            Ok(None) => Ok(None),
            Ok(Some(raw_packet)) => {
                let payload_type = raw_packet
                    .header()
                    .payload_type()
                    .ok_or(PacketFromSliceError::UnknownPayloadType)?;

                let (payload, payload_size) = Payload::<'a>::payload_from_json_slice(
                    payload_type,
                    raw_packet.payload_bytes(),
                )?;

                let packet_length = Header::size() + payload_size;

                Ok(Some((Packet { payload }, packet_length)))
            }
        }
    }
}

/// Error returned by [`Packet::write_to`].
#[derive(Debug, From, thiserror::Error)]
pub enum PacketWriteError {
    /// Failed to write raw packet.
    #[error("Failed to write raw packet")]
    RawPacket(RawPacketWriteError),
}

/// Error returned by [`Packet::maybe_packet_from_prefix`].
#[derive(Debug, From, thiserror::Error)]
pub enum PacketFromSliceError {
    /// Invalid raw packet.
    #[error("Invalid raw packet")]
    RawPacket(RawPacketFromSliceError),
    /// Unknown payload type.
    #[error("Unknown payload type")]
    UnknownPayloadType,
    /// Invalid payload.
    #[error("Invalid payload")]
    Payload(PayloadFromSliceError),
}

#[cfg(test)]
mod test {
    use crate::payload_content::{DeviceConfig, PayloadContent};

    use super::*;

    #[test]
    fn encode_decode() {
        let buf = &mut [0; 100];

        let packet = Packet::new_raw(Payload::new_raw(PayloadContent::DeviceConfig(
            DeviceConfig {
                sequence_number: 12,
                config: "config",
            },
        )));

        let written = packet.write_to(buf).expect("Must be ok");

        let (reconstructed, read) = Packet::maybe_packet_from_prefix(buf)
            .expect("Must be ok")
            .expect("Must be some");

        assert_eq!(written, read);
        assert_eq!(reconstructed, packet);
    }
}
