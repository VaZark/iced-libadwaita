//! The sample data of the chart playgrounds (see `plays.rs`).

use iced_adaptive_design as iad;

use iad::widget::{Sankey, Series, TreeValue};

pub const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

pub fn sales() -> [Series; 3] {
    [
        Series::new("Web", [12.0, 19.0, 15.0, 25.0, 22.0, 30.0]),
        Series::new("Store", [8.0, 11.0, 14.0, 10.0, 16.0, 18.0]),
        Series::new("Partners", [4.0, 6.0, 5.0, 9.0, 12.0, 11.0]),
    ]
}

/// Browser shares, for the pie.
pub fn browsers() -> [(&'static str, f32); 5] {
    [
        ("Chrome", 62.0),
        ("Firefox", 21.0),
        ("Safari", 9.0),
        ("Edge", 6.0),
        ("Other", 2.0),
    ]
}

/// A conversion funnel.
pub fn stages() -> [(&'static str, f32); 5] {
    [
        ("Visits", 5000.0),
        ("Sign-ups", 2400.0),
        ("Trials", 1100.0),
        ("Paid", 420.0),
        ("Renewed", 300.0),
    ]
}

/// A household budget, for the sankey.
pub fn budget() -> Sankey {
    Sankey::new()
        .link("Salary", "Income", 5000.0)
        .link("Dividends", "Income", 600.0)
        .link("Income", "Housing", 1800.0)
        .link("Income", "Food", 900.0)
        .link("Income", "Transport", 500.0)
        .link("Income", "Savings", 2400.0)
        .link("Savings", "Investments", 1500.0)
        .link("Savings", "Cash", 900.0)
}

/// Disk usage, for the sunburst.
pub fn disk() -> TreeValue {
    TreeValue::branch(
        "Disk",
        [
            TreeValue::branch(
                "Media",
                [
                    TreeValue::leaf("Photos", 40.0),
                    TreeValue::leaf("Video", 90.0),
                    TreeValue::leaf("Music", 25.0),
                ],
            ),
            TreeValue::branch(
                "Code",
                [
                    TreeValue::branch(
                        "Rust",
                        [
                            TreeValue::leaf("src", 12.0),
                            TreeValue::leaf("target", 60.0),
                        ],
                    ),
                    TreeValue::leaf("Web", 18.0),
                ],
            ),
            TreeValue::leaf("System", 35.0),
        ],
    )
}
