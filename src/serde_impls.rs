use std::marker::PhantomData;
use std::fmt;
use serde::de::{SeqAccess, Visitor};
use serde::ser::{SerializeSeq};
use serde::{self, Serialize, Deserialize, Serializer, Deserializer};
use {ArrayVec, ArrayString};
use array::Array;

impl<A: Array> Serialize for ArrayVec<A>
    where A::Item: Serialize
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = try!(serializer.serialize_seq(Some(self.len())));
        for i in self.iter() {
            try!(seq.serialize_element(i));
        }
        seq.end()
    }
}

struct ArrayVecVisitor<'de, A: 'de>(PhantomData<&'de A>);

impl<'de, A: Array> Visitor<'de> for ArrayVecVisitor<'de, A>
    where A::Item: Deserialize<'de>
{
    type Value = ArrayVec<A>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A sequence.")
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        use serde::de::Error;
        let mut ret = ArrayVec::<A>::new();
        while let Some(elem) = try!(seq.next_element()) {
            if ret.push(elem).is_some() {
                return Err(S::Error::custom("Too many elements"));
            }
        }
        Ok(ret)
    }
}

impl<'de, A: 'de+Array> Deserialize<'de> for ArrayVec<A> 
    where A::Item: Deserialize<'de>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(ArrayVecVisitor::<A>(PhantomData))
    }
}


impl<A: Array<Item=u8>> Serialize for ArrayString<A> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self)
    }
}

struct ArrayStringVisitor<'de, A: 'de>(PhantomData<&'de A>);

impl<'de, A: Array<Item=u8>> Visitor<'de> for ArrayStringVisitor<'de, A>{
    type Value = ArrayString<A>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("A string")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        ArrayString::from(v).map_err(|_| E::custom("String too large"))
    }
}

impl<'de, A: Array<Item=u8>+'de> Deserialize<'de> for ArrayString<A> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(ArrayStringVisitor(PhantomData))
    }
}
