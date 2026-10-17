use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;

pub type Members = BTreeMap<identity::PeerId, discovery::Member>;
