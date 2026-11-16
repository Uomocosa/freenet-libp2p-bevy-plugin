use crate::testing;

pub fn free_display_name() -> Result<String, String> {
    for number in testing::harness::XVFB_FIRST_DISPLAY..=testing::harness::XVFB_LAST_DISPLAY {
        let name = format!(":{number}");
        if !testing::harness::E2EDisplay::is_display_taken(&name) {
            return Ok(name);
        }
    }
    Err(format!(
        "e2e display: every display in :{first}..=:{last} is taken; set {reuse} to reuse one",
        first = testing::harness::XVFB_FIRST_DISPLAY,
        last = testing::harness::XVFB_LAST_DISPLAY,
        reuse = testing::harness::E2E_DISPLAY_ENV
    ))
}

#[cfg(test)]
mod tests {
    use super::free_display_name;

    #[test]
    fn test_usage() {
        let free = free_display_name().expect("a free display in range");
        assert!(free.starts_with(':'));
        assert!(free.len() > 1);
    }
}
