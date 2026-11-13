use crate::testing;

pub fn parsed_script<Step>(
    args: &testing::E2EArgs<Step>,
) -> Result<Option<Vec<testing::TimedScriptStep<Step>>>, String>
where
    Step: testing::traits::ScriptStep + Clone + Send + Sync + 'static,
{
    args.script
        .clone()
        .map(|raw| {
            raw.iter()
                .map(|text| testing::TimedScriptStep::<Step>::try_from(text.as_str()))
                .collect::<Result<Vec<_>, String>>()
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::parsed_script;
    use crate::testing;

    #[test]
    fn test_usage() {
        let stay = testing::E2EArgs::<testing::E2EStep> {
            script: Some(vec!["stay".to_string()]),
            result_json: None,
            marker: PhantomData,
        };
        assert_eq!(
            parsed_script(&stay).map(|steps| steps.map(|v| v.len())),
            Ok(Some(1))
        );
        let none = testing::E2EArgs::<testing::E2EStep> {
            script: None,
            result_json: None,
            marker: PhantomData,
        };
        assert_eq!(parsed_script(&none), Ok(None));
        let bad = testing::E2EArgs::<testing::E2EStep> {
            script: Some(vec!["fly".to_string()]),
            result_json: None,
            marker: PhantomData,
        };
        assert!(parsed_script(&bad).is_err());
    }
}
