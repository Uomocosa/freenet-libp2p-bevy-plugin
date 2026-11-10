use std::path::PathBuf;

use clap::Args;

use crate::testing;

#[derive(Debug, Clone, Args)]
pub struct E2EArgs<Step>
where
    Step: testing::traits::ScriptStep + Clone + Send + Sync + 'static,
{
    #[arg(long, value_delimiter = ',')]
    pub script: Option<Vec<String>>,
    #[arg(long)]
    pub result_json: Option<PathBuf>,
    #[arg(skip)]
    pub marker: std::marker::PhantomData<Step>,
}

#[rustfmt::skip]
impl<Step> E2EArgs<Step>
where
    Step: testing::traits::ScriptStep + Clone + Send + Sync + 'static,
{
    pub fn parsed_script(&self) -> Result<Option<Vec<testing::TimedScriptStep<Step>>>, String> { self.script.clone().map(|raw| raw.iter().map(|text| testing::TimedScriptStep::<Step>::try_from(text.as_str())).collect::<Result<Vec<_>, String>>()).transpose() }
}

#[cfg(test)]
mod tests {
    use super::E2EArgs;
    use crate::testing;
    use clap::Parser;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ToyStep;

    impl testing::traits::ScriptStep for ToyStep {
        const STEP_NAMES: &'static str = "stay";

        fn parse_verb(verb: &str, _arg: &str) -> Option<Result<Self, String>> {
            match verb {
                "stay" => Some(Ok(Self)),
                _ => None,
            }
        }

        fn is_wait(&self) -> bool {
            false
        }
    }

    #[derive(Debug, Parser)]
    struct Probe {
        #[command(flatten)]
        e2e: E2EArgs<ToyStep>,
    }

    #[test]
    fn test_usage() {
        let probe = Probe::try_parse_from(["probe", "--script", "stay"]).expect("script");
        assert_eq!(probe.e2e.script.clone().map(|steps| steps.len()), Some(1));
        let parsed = probe.e2e.parsed_script().expect("parsed").expect("some");
        assert_eq!(parsed.len(), 1);
    }
}
