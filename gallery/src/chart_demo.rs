//! The sample data of the chart playgrounds (see `plays.rs`).

use iced_adaptive_design as iad;

use iad::widget::{BubblePoint, BubbleSeries, Sankey, Series, TreeValue};

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

/// Spend, sales and market size by region, for the bubble chart.
pub fn markets() -> [BubbleSeries; 3] {
    let b = BubblePoint::new;
    [
        BubbleSeries::new(
            "Europe",
            [
                b(12.0, 30.0, 40.0),
                b(25.0, 48.0, 90.0),
                b(40.0, 61.0, 30.0),
                b(18.0, 22.0, 15.0),
            ],
        ),
        BubbleSeries::new(
            "Americas",
            [
                b(30.0, 40.0, 120.0),
                b(52.0, 75.0, 60.0),
                b(8.0, 15.0, 20.0),
            ],
        ),
        BubbleSeries::new(
            "Asia",
            [
                b(45.0, 80.0, 150.0),
                b(20.0, 55.0, 70.0),
                b(60.0, 66.0, 35.0),
                b(35.0, 28.0, 25.0),
            ],
        ),
    ]
}

/// Revenue by region (outer) and by product (inner), for the donut.
type Shares<const N: usize> = [(&'static str, f32); N];

pub fn revenue() -> (Shares<4>, Shares<3>) {
    (
        [
            ("North", 42.0),
            ("South", 28.0),
            ("East", 18.0),
            ("West", 12.0),
        ],
        [("Hardware", 55.0), ("Software", 30.0), ("Services", 15.0)],
    )
}

/// Revenue and costs as bars, a target as a line and a trend as an area, for the combo chart.
pub fn quarters() -> ([&'static str; 6], [Series; 4]) {
    (
        ["Q1", "Q2", "Q3", "Q4", "Q5", "Q6"],
        [
            Series::new("Trend", [30.0, 38.0, 44.0, 50.0, 58.0, 66.0]),
            Series::new("Revenue", [40.0, 55.0, 38.0, 62.0, 70.0, 64.0]),
            Series::new("Costs", [22.0, 28.0, 25.0, 31.0, 36.0, 34.0]),
            Series::new("Target", [45.0, 50.0, 55.0, 60.0, 65.0, 70.0]),
        ],
    )
}
