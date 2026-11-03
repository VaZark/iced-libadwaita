//! The chart widgets: line, bar, pie, funnel, sankey and sunburst, with their variants. The
//! charts own their data and have no events, so the pages are plain functions.

use iced_adaptive_design as iad;

use iad::primitives::histogram::Histogram;
use iad::widget::{
    self as w, BarChart, BarLayout, Curve, Funnel, FunnelShape, LineChart, PieChart,
    PreferencesGroup, Sankey, Series, Sunburst, TreeValue,
};
use iad::{Element, Metrics};

fn group<'a, M: 'a>(
    title: &'static str,
    desc: &'static str,
    m: Metrics,
    chart: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    PreferencesGroup::<M>::new()
        .title(title)
        .description(desc)
        .metrics(m)
        .push(w::card(chart.into()))
        .into()
}

const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

fn sales() -> [Series; 3] {
    [
        Series::new("Web", [12.0, 19.0, 15.0, 25.0, 22.0, 30.0]),
        Series::new("Store", [8.0, 11.0, 14.0, 10.0, 16.0, 18.0]),
        Series::new("Partners", [4.0, 6.0, 5.0, 9.0, 12.0, 11.0]),
    ]
}

fn page<'a, M: 'a>(m: Metrics, groups: Vec<Element<'a, M>>) -> Element<'a, M> {
    iced::widget::column(groups).spacing(m.group_spacing).into()
}

pub fn line<'a, M: 'a>(m: Metrics) -> Element<'a, M> {
    page(
        m,
        vec![
            group(
                "Lines",
                "Hover for the values at a category.",
                m,
                LineChart::new(MONTHS, sales()),
            ),
            group(
                "Smooth",
                "A monotone curve never overshoots the data.",
                m,
                LineChart::new(MONTHS, sales()).curve(Curve::Monotone),
            ),
            group(
                "Steps",
                "",
                m,
                LineChart::new(MONTHS, [sales()[0].clone()]).curve(Curve::Step),
            ),
            group(
                "Area",
                "",
                m,
                LineChart::new(MONTHS, [sales()[0].clone(), sales()[1].clone()])
                    .curve(Curve::Monotone)
                    .area(true),
            ),
            group(
                "Stacked area",
                "",
                m,
                LineChart::new(MONTHS, sales())
                    .curve(Curve::Monotone)
                    .stacked(true),
            ),
            group(
                "100% area",
                "",
                m,
                LineChart::new(MONTHS, sales()).normalized(true),
            ),
        ],
    )
}

pub fn bar<'a, M: 'a>(m: Metrics) -> Element<'a, M> {
    let h = Histogram::from_values(
        (0..300).map(|i| 100.0 * ((i as f32 * 0.618_034).fract()).powi(2)),
        0.0,
        100.0,
        12,
    );
    page(
        m,
        vec![
            group("Grouped", "", m, BarChart::new(MONTHS, sales())),
            group(
                "Stacked",
                "",
                m,
                BarChart::new(MONTHS, sales()).layout(BarLayout::Stacked),
            ),
            group(
                "100%",
                "",
                m,
                BarChart::new(MONTHS, sales()).layout(BarLayout::Normalized),
            ),
            group(
                "Horizontal",
                "",
                m,
                BarChart::new(MONTHS, sales())
                    .horizontal(true)
                    .height(300.0),
            ),
            group(
                "Histogram",
                "Counts of values per bin: the same bars, touching.",
                m,
                BarChart::histogram(&h),
            ),
        ],
    )
}

pub fn pie<'a, M: 'a>(m: Metrics) -> Element<'a, M> {
    let items = [
        ("Chrome", 62.0),
        ("Firefox", 21.0),
        ("Safari", 9.0),
        ("Edge", 6.0),
        ("Other", 2.0),
    ];
    page(
        m,
        vec![
            group("Pie", "", m, PieChart::new(items)),
            group(
                "Donut",
                "",
                m,
                PieChart::new(items).donut(0.6).center("Browsers"),
            ),
        ],
    )
}

pub fn funnel<'a, M: 'a>(m: Metrics) -> Element<'a, M> {
    let items = [
        ("Visits", 5000.0),
        ("Sign-ups", 2400.0),
        ("Trials", 1100.0),
        ("Paid", 420.0),
        ("Renewed", 300.0),
    ];
    page(
        m,
        vec![
            group(
                "Funnel",
                "Hover for the share of the first stage and of the previous one.",
                m,
                Funnel::new(items),
            ),
            group(
                "Blocks",
                "",
                m,
                Funnel::new(items).shape(FunnelShape::Blocks),
            ),
        ],
    )
}

pub fn sankey<'a, M: 'a>(m: Metrics) -> Element<'a, M> {
    let s = Sankey::new()
        .link("Salary", "Income", 5000.0)
        .link("Dividends", "Income", 600.0)
        .link("Income", "Housing", 1800.0)
        .link("Income", "Food", 900.0)
        .link("Income", "Transport", 500.0)
        .link("Income", "Savings", 2400.0)
        .link("Savings", "Investments", 1500.0)
        .link("Savings", "Cash", 900.0);
    page(m, vec![group("Sankey", "Hover a node or a flow.", m, s)])
}

pub fn sunburst<'a, M: 'a>(m: Metrics) -> Element<'a, M> {
    let t = TreeValue::branch(
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
    );
    page(
        m,
        vec![group(
            "Sunburst",
            "Each ring is a level; the middle names what the pointer is over.",
            m,
            Sunburst::new(t),
        )],
    )
}
