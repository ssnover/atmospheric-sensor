#![cfg_attr(not(feature = "use_std"), no_std)]

pub mod msg;
pub use msg::*;

#[cfg(not(feature = "use_std"))]
pub fn encode<'a, 'b>(
    msg: &'b Message,
    buffer: &'a mut [u8],
) -> Result<&'a mut [u8], postcard::Error> {
    postcard::to_slice_cobs(msg, buffer)
}

pub fn encode_body<'a, 'b, T>(
    msg: &'b T,
    buffer: &'a mut [u8],
) -> Result<&'a mut [u8], postcard::Error>
where
    T: serde::Serialize,
{
    postcard::to_slice(msg, buffer)
}

#[cfg(feature = "use_std")]
pub fn encode(msg: &Message) -> Result<Vec<u8>, postcard::Error> {
    postcard::to_stdvec_cobs(msg)
}

pub fn decode_packet<'a>(buffer: &'a mut [u8]) -> Result<(Message, &'a mut [u8]), postcard::Error> {
    postcard::take_from_bytes_cobs::<Message>(buffer)
}

pub fn from_buffer<'a, T>(body: &'a [u8]) -> Result<T, postcard::Error>
where
    T: serde::Deserialize<'a>,
{
    postcard::from_bytes::<T>(body)
}

mod tests {
    use crate::*;

    #[test]
    fn decode_co2_data() {
        let msg = ReportCO2Data { measurement: 300. };
        let mut body_buffer = [0u8; 32];
        let msg = Message {
            hdr: Header {
                version: 0x00,
                id: 0x00,
                msg_type: MessageType::ReportCO2Data,
            },
            msg: &encode_body(&msg, &mut body_buffer).unwrap()[..],
        };
        let mut encode_buf = [0u8; 32];
        let encode_result = encode(&msg, &mut encode_buf);
        assert!(encode_result.is_ok());

        let mut encoded_packet = encode_result.unwrap();
        let decode_result = decode_packet(&mut encoded_packet);

        assert!(decode_result.is_ok());

        let (msg, remaining) = decode_result.unwrap();
        assert_eq!(msg.hdr.msg_type, MessageType::ReportCO2Data);
        assert_eq!(remaining, &[0]);

        let res = from_buffer::<ReportCO2Data>(msg.msg);
        assert!(res.is_ok());
        let data = res.unwrap();
        assert_eq!(data.measurement, 300.);
    }
}
