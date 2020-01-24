use dat_network_protocol as proto;

pub use simple_message_channels::{Message, Reader, Writer};

#[non_exhaustive]
#[derive(Debug)]
pub enum DatMessage {
    Open(proto::Open),
    Options(proto::Options),
    Status(proto::Status),
    Have(proto::Have),
    Unhave(proto::Unhave),
    Want(proto::Want),
    Unwant(proto::Unwant),
    Request(proto::Request),
    Cancel(proto::Cancel),
    Data(proto::Data),
    Close(proto::Close),
}

type ParseResult = Result<DatMessage, protobuf::ProtobufError>;

pub trait MessageExt {
    fn parse(&self) -> ParseResult;
}

impl MessageExt for Message {
    fn parse(&self) -> ParseResult {
        match self.typ {
            0 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Open),
            1 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Options),
            2 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Status),
            3 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Have),
            4 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Unhave),
            5 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Want),
            6 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Unwant),
            7 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Request),
            8 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Cancel),
            9 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Data),
            10 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Close),
            _ => panic!("Uknonw message"), // TODO proper error handling
        }
    }
}
