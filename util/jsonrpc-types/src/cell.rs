use crate::blockchain::{CellOutput, OutPoint};
use ckb_core::cell::CellStatus;
use ckb_core::script::Script;
use ckb_core::Capacity;
use serde_derive::Serialize;

// This is used as return value of get_cells_by_type_hash RPC:
// it contains both OutPoint data used for referencing a cell, as well as
// cell's own data such as lock and capacity
#[derive(Serialize)]
pub struct CellOutputWithOutPoint {
    pub out_point: OutPoint,
    pub capacity: Capacity,
    pub lock: Script,
}

#[derive(Serialize)]
pub struct CellWithStatus {
    pub cell: Option<CellOutput>,
    pub status: String,
}

impl From<CellStatus> for CellWithStatus {
    fn from(status: CellStatus) -> Self {
        let (cell, status) = match status {
            CellStatus::Live(cell) => (Some(cell), "live"),
            CellStatus::Dead => (None, "dead"),
            CellStatus::Unknown => (None, "unknown"),
        };
        Self {
            cell: cell.map(Into::into),
            status: status.to_string(),
        }
    }
}
