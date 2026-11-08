use super::*;
use iced::advanced::{
    Shell, clipboard, layout,
    renderer::{Headless, Renderer},
    widget::Tree,
};
use iced::{Event, Point, Rectangle, Size, mouse};

fn frame(
    gallery: &Gallery,
    tree: &mut Tree,
    renderer: &mut iced::Renderer,
    now: Instant,
) -> Vec<u8> {
    let size = Size::new(1100.0, 900.0);
    let viewport = Rectangle::new(Point::ORIGIN, size);
    let mut view = gallery.view(gallery.main);
    tree.diff(view.as_widget());
    let limits = layout::Limits::new(Size::ZERO, size);
    let node = view.as_widget_mut().layout(tree, renderer, &limits);
    let mut messages = vec![];
    let mut shell = Shell::new(&mut messages);
    view.as_widget_mut().update(
        tree,
        &Event::Window(window::Event::RedrawRequested(now)),
        layout::Layout::new(&node),
        mouse::Cursor::Unavailable,
        renderer,
        &mut clipboard::Null,
        &mut shell,
        &viewport,
    );
    let node = if shell.is_layout_invalid() {
        view.as_widget_mut().layout(tree, renderer, &limits)
    } else {
        node
    };
    let theme = gallery.build_theme();
    renderer.reset(viewport);
    view.as_widget().draw(
        tree,
        renderer,
        &theme,
        &iced::advanced::renderer::Style {
            text_color: theme.semantic.window.fg,
        },
        layout::Layout::new(&node),
        mouse::Cursor::Unavailable,
        &viewport,
    );
    renderer.screenshot(Size::new(1100, 900), 1.0, theme.semantic.window.bg)
}

#[test]
fn disclosure_does_not_flash_gallery_chrome_in_either_scheme() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let _runtime = runtime.enter();
    let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.67),
        Some(&std::env::var("GALLERY_TEST_RENDERER").unwrap_or_else(|_| "tiny-skia".into())),
    ))
    .unwrap();
    let (mut gallery, _) = Gallery::new();
    for scheme in [Scheme::Light, Scheme::Dark] {
        gallery.scheme = scheme;
        for detail in [Detail::TreeRow, Detail::Accordion] {
            gallery.page = detail.page();
            gallery.detail = Some(detail);
            let mut tree = Tree::empty();
            let start = Instant::now();
            let _ = frame(&gallery, &mut tree, &mut renderer, start);
            let baseline = frame(
                &gallery,
                &mut tree,
                &mut renderer,
                start + Duration::from_secs(2),
            );
            if let Ok(dir) = std::env::var("GALLERY_FLASH_SHOTS") {
                std::fs::create_dir_all(&dir).unwrap();
                image::save_buffer(
                    format!("{dir}/{scheme:?}-{detail:?}-before.png"),
                    &baseline,
                    1100,
                    900,
                    image::ColorType::Rgba8,
                )
                .unwrap();
            }
            for (cycle, open) in [false, true, false].into_iter().enumerate() {
                let message = if detail == Detail::TreeRow {
                    Message::Play(detail, playground::Msg::Bool("expanded", open))
                } else {
                    Message::AccSingle(if open { vec![0] } else { vec![] })
                };
                let _ = gallery.update(message);
                for ms in [0, 16, 70, 160, 300, 1000] {
                    let pixels = frame(
                        &gallery,
                        &mut tree,
                        &mut renderer,
                        start + Duration::from_millis(3000 + cycle as u64 * 2000 + ms),
                    );
                    let mut changes = 0;
                    for y in 0..900 {
                        for x in 0..1100 {
                            if y < 44 || x < 170 {
                                let at = (y * 1100 + x) * 4;
                                changes += usize::from(pixels[at..at + 4] != baseline[at..at + 4]);
                            }
                        }
                    }
                    if let Ok(dir) = std::env::var("GALLERY_FLASH_SHOTS") {
                        image::save_buffer(
                            format!("{dir}/{scheme:?}-{detail:?}-{open}-{ms}.png"),
                            &pixels,
                            1100,
                            900,
                            image::ColorType::Rgba8,
                        )
                        .unwrap();
                    }
                    assert_eq!(
                        changes, 0,
                        "{scheme:?} {detail:?} open={open} ms={ms}: chrome changed"
                    );
                }
            }
        }
    }
}

#[test]
fn review_application_forms() {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let _runtime = runtime.enter();
    let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(iced::Font::DEFAULT, iced::Pixels(14.67), Some("tiny-skia"))).unwrap();
    let (mut gallery, _) = Gallery::new();
    for scheme in [Scheme::Light, Scheme::Dark] {
        gallery.scheme = scheme;
        for detail in [Detail::Entries, Detail::Form] {
            gallery.page = detail.page(); gallery.detail = Some(detail);
            let mut tree = Tree::empty();
            let start = Instant::now();
            let _ = frame(&gallery, &mut tree, &mut renderer, start);
            let pixels = frame(&gallery, &mut tree, &mut renderer, start + Duration::from_secs(2));
            image::save_buffer(format!("/tmp/iad-{scheme:?}-{detail:?}.png"), &pixels, 1100, 900, image::ColorType::Rgba8).unwrap();
        }
    }
}
