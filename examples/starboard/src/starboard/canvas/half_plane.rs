#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HalfPlane {
    pub a: i64,
    pub b: i64,
    pub k: i64,
}

impl HalfPlane {
    #[must_use]
    pub(crate) fn origin_sectors(vertices: &[[i64; 2]], radius: i64) -> Vec<Vec<Self>> {
        let len = vertices.len();
        (0..len)
            .filter_map(|idx| {
                let [ax0, ay0] = *vertices.get(idx)?;
                let next = idx.saturating_add(1).checked_rem(len)?;
                let [bx0, by0] = *vertices.get(next)?;
                let (ax, ay) = (ax0.saturating_mul(radius), ay0.saturating_mul(radius));
                let (bx, by) = (bx0.saturating_mul(radius), by0.saturating_mul(radius));
                Some(vec![
                    Self {
                        a: ay.saturating_neg(),
                        b: ax,
                        k: 0,
                    },
                    Self {
                        a: by.saturating_sub(ay).saturating_neg(),
                        b: bx.saturating_sub(ax),
                        k: bx
                            .saturating_sub(ax)
                            .saturating_mul(ay)
                            .saturating_neg()
                            .saturating_add(by.saturating_sub(ay).saturating_mul(ax)),
                    },
                    Self {
                        a: by,
                        b: bx.saturating_neg(),
                        k: 0,
                    },
                ])
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::HalfPlane;

    #[test]
    fn test_usage() {
        let sectors = HalfPlane::origin_sectors(&[[0, -1000], [866, 500], [-866, 500]], 1);
        assert_eq!(sectors.len(), 3);
        assert!(sectors.iter().all(|sector| sector.len() == 3));
    }
}
