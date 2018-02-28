use transaction::{TxInGen, TxInToKey, TxInToScript, TxInToScriptHash};
use format::{
    Deserialize,
    DeserializerStream,
    Error,
    Serialize,
    SerializerStream
};

const GEN: u8 = 0xff;
const TO_KEY: u8 = 2;
const TO_SCRIPT: u8 = 0;
const TO_SCRIPT_HASH: u8 = 1;

/// Transaction input.
#[derive(Debug)]
pub enum TxIn {
    Gen(TxInGen),
    ToKey(TxInToKey),
    ToScript(TxInToScript),
    ToScriptHash(TxInToScriptHash),
}

impl TxIn {
    pub fn signature_size(&self) -> usize {
        match *self {
            TxIn::Gen(_) => 0,
            TxIn::ToKey(ref tx) => tx.key_offsets.len(),
            TxIn::ToScript(_) => 0,
            TxIn::ToScriptHash(_) => 0,
        }
    }
}

impl Deserialize for TxIn {
    fn deserialize(mut deserializer: DeserializerStream) -> Result<Self, Error> {
        let tag = deserializer.get_u8()?;
        let target = match tag {
            GEN => {
                TxIn::Gen(deserializer.get_deserializable()?)
            },
            TO_KEY => {
                TxIn::ToKey(deserializer.get_deserializable()?)
            },
            TO_SCRIPT => {
                TxIn::ToScript(deserializer.get_deserializable()?)
            },
            TO_SCRIPT_HASH => {
                TxIn::ToScriptHash(deserializer.get_deserializable()?)
            },
            n => return Err(Error::custom(format!("unknown variant tag: {:X}", n))),
        };

        Ok(target)
    }
}

impl Serialize for TxIn {
    fn serialize(&self, mut serializer: SerializerStream) {
        match *self {
            TxIn::Gen(ref v) => {
                serializer.put_u8(GEN);
                serializer.put_serializable(v);
            },
            TxIn::ToKey(ref v) => {
                serializer.put_u8(TO_KEY);
                serializer.put_serializable(v);
            },
            TxIn::ToScript(ref v) => {
                serializer.put_u8(TO_SCRIPT);
                serializer.put_serializable(v);
            },
            TxIn::ToScriptHash(ref v) => {
                serializer.put_u8(TO_SCRIPT_HASH);
                serializer.put_serializable(v);
            },
        }
    }
}