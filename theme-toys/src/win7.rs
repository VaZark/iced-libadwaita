//! Windows 7: an Explorer window. Navigation pane (tree), a details list, a search box, a
//! command bar, a preview pane and a notification.

use crate::common::*;
use iad::components as cmp;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;
use iad::widgets as w;

pub const TITLE: &str = "Libraries - Documents";
const FILES: [(&str, &str, &str, &str); 6] = [
    ("Budget 2011.xlsx", "9/14/2011", "Microsoft Excel", "84 KB"),
    ("Holiday.jpg", "8/2/2011", "JPEG image", "2.4 MB"),
    ("Notes.txt", "7/30/2011", "Text Document", "3 KB"),
    ("Report.docx", "7/12/2011", "Microsoft Word", "212 KB"),
    ("Song.mp3", "6/20/2011", "MP3 Format Sound", "4.1 MB"),
    ("Setup.exe", "6/1/2011", "Application", "18 MB"),
];

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let commands = row![
        cmp::raised_button(cmp::typography::label("Organize")).on_press(Msg::Status("Organize".into())),
        cmp::raised_button(cmp::typography::label("Share with")).on_press(Msg::Status("Share".into())),
        cmp::raised_button(cmp::typography::label("New folder")).on_press(Msg::Toast(Some("Folder created".into()))),
        space().width(Length::Fill),
        cmp::search_bar(t.text.as_str(), Msg::Text).placeholder("Search Documents").width(220),
    ]
    .spacing(m.space_s)
    .align_y(Alignment::Center)
    .padding(m.space_s);
    let header = row![caption("Name"), space().width(Length::Fill), caption("Date modified"), caption("Type"), caption("Size")].spacing(m.gap).padding([m.space_xxs, m.space_m]);
    let rows = column(FILES.iter().enumerate().map(|(i, (n, d, ty, sz))| {
        let line = row![Icon::Document.view(16.0), label(*n), space().width(Length::Fill), label(*d), label(*ty), label(*sz)]
            .spacing(m.gap)
            .align_y(Alignment::Center);
        let b = if i == t.pick { cmp::raised_button(line) } else { cmp::flat_button(line) };
        El::from(b.width(Length::Fill).on_press(Msg::Pick(i)))
    }));
    let list = column![El::from(header), pane(rows)];
    let preview = container(w::card(column![Icon::Document.view(48.0), label(FILES[t.pick.min(5)].0), caption(FILES[t.pick.min(5)].2)].spacing(m.space_s).align_x(Alignment::Center)))
        .width(190);
    let body = row![container(nav_tree(t)).width(200).padding(m.space_s), El::from(list.width(Length::Fill)), container(preview).padding(m.space_s)].height(Length::Fill);
    let content = column![El::from(commands), El::from(body), status_bar(t.status.clone(), format!("{} items", FILES.len()))];
    let dialog: El<'_> = w::AlertDialog::new("Delete File").body("Are you sure you want to move this file to the Recycle Bin?")
        .response("No", w::ResponseAppearance::Default, Msg::Dialog(false))
        .response("Yes", w::ResponseAppearance::Destructive, Msg::Dialog(false)).into();
    window(t, TITLE, overlays(t, content.into(), dialog))
}
