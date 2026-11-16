pub fn scale_from_env() -> Option<f64> {
    std::env::var("E2E_TIME_SCALE")
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|scale| scale.is_finite() && *scale > 0.0)
}

// no test_usage necessary
