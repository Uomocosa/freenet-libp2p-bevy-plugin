// no test_usage necessary
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeployPolicy {
    FetchOnly,
    #[default]
    FetchOrDeploy,
}
