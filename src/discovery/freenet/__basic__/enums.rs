// no test_usage necessary
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployPolicy {
    FetchOnly,
    FetchOrDeploy,
}
