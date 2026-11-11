use freenet_stdlib::prelude::UpdateData;

pub fn decode_update(data: &UpdateData<'static>) -> Option<Vec<u8>> {
    let raw_bytes: Option<&[u8]> = match data {
        UpdateData::State(state) | UpdateData::StateAndDelta { state, .. } => Some(state.as_ref()),
        UpdateData::Delta(delta) => Some(delta.as_ref()),
        _ => None,
    };
    raw_bytes.map(<[u8]>::to_vec)
}

#[cfg(test)]
mod tests {
    use super::decode_update;
    use freenet_stdlib::prelude::{ContractInstanceId, State, StateDelta, UpdateData};

    #[test]
    fn test_usage() {
        assert_eq!(
            decode_update(&UpdateData::State(State::from(vec![1u8, 2, 3]))),
            Some(vec![1u8, 2, 3])
        );
        assert_eq!(
            decode_update(&UpdateData::Delta(StateDelta::from(vec![4u8]))),
            Some(vec![4u8])
        );
        assert_eq!(
            decode_update(&UpdateData::RelatedDelta {
                related_to: ContractInstanceId::new([0; 32]),
                delta: StateDelta::from(vec![4u8]),
            }),
            None
        );
    }
}
