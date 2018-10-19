use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use std::cell::Cell;
use std::rc::Rc;

// serde does not support serialize / deserialize rc so we use our own deserializer
pub fn deserialize_rc_cell<'de, D, T>(deserializer: D) -> Result<Rc<Cell<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Copy,
{
    Ok(Rc::new(Cell::deserialize(deserializer).unwrap()))
}

// serde does not support serialize / deserialize rc so we use our own serializer
pub fn serialize_rc_cell<T, S>(
    rc_cell: &Rc<Cell<T>>,
    serializer: S,
) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
where
    S: Serializer,
    T: Serialize + Copy,
{
    rc_cell.serialize(serializer)
}
