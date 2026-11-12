use crate::starboard::canvas::CellBox;

/// Side of one bin, in cells. Each bin lists the pairs whose box overlaps it.
const BIN_SIDE: i64 = 4;

/// The pair indices per bin over a rectangle of cells, so each cell looks up only nearby pairs.
pub struct PairBins {
    pub origin_x: i64,
    pub origin_y: i64,
    pub columns: i64,
    pub bins: Vec<Vec<usize>>,
}

impl PairBins {
    /// Registers every box under each bin it overlaps; `union` must contain every box.
    #[must_use]
    pub fn new(boxes: &[CellBox], union: CellBox) -> Self {
        let origin_x = union.low_x;
        let origin_y = union.low_y;
        let columns = union
            .high_x
            .saturating_sub(origin_x)
            .div_euclid(BIN_SIDE)
            .saturating_add(1);
        let rows = union
            .high_y
            .saturating_sub(origin_y)
            .div_euclid(BIN_SIDE)
            .saturating_add(1);
        let mut bins = vec![Vec::new(); usize::try_from(columns.saturating_mul(rows)).unwrap_or(0)];
        for (index, cell_box) in boxes.iter().enumerate() {
            let first_column = cell_box.low_x.saturating_sub(origin_x).div_euclid(BIN_SIDE);
            let last_column = cell_box
                .high_x
                .saturating_sub(origin_x)
                .div_euclid(BIN_SIDE);
            let first_row = cell_box.low_y.saturating_sub(origin_y).div_euclid(BIN_SIDE);
            let last_row = cell_box
                .high_y
                .saturating_sub(origin_y)
                .div_euclid(BIN_SIDE);
            for row in first_row..=last_row {
                for column in first_column..=last_column {
                    let slot = row.saturating_mul(columns).saturating_add(column);
                    if let Some(bin) = usize::try_from(slot).ok().and_then(|at| bins.get_mut(at)) {
                        bin.push(index);
                    }
                }
            }
        }
        Self {
            origin_x,
            origin_y,
            columns,
            bins,
        }
    }

    /// The pairs whose box overlaps the bin that holds the cell `(x, y)`, in index order.
    #[must_use]
    pub fn candidates(&self, x: i64, y: i64) -> &[usize] {
        let column = x.saturating_sub(self.origin_x).div_euclid(BIN_SIDE);
        let row = y.saturating_sub(self.origin_y).div_euclid(BIN_SIDE);
        usize::try_from(row.saturating_mul(self.columns).saturating_add(column))
            .ok()
            .and_then(|slot| self.bins.get(slot))
            .map_or(&[], Vec::as_slice)
    }
}
