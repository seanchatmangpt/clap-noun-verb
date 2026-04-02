pub mod report;

pub use report::{
    ReportError, SalesRecord, SalesStats, CategoryStats, ProductStats,
    ReportFormat, aggregate_sales, format_report,
};
