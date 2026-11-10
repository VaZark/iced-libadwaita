//! Arrow keys that leave a text field at its start or end, to move between the token groups
//! around it. The field keeps focus and its cursor; the arrow is captured only when it would
//! have gone nowhere inside the field.
use iced::Event;
use iced::advanced::widget::Tree;

pub(crate) type InputState =
    iced::widget::text_input::State<<iced::Renderer as iced::advanced::text::Renderer>::Paragraph>;
/// Looks at the field's tree and an event, and says which message (if any) the event means.
pub(crate) type Navigation<'a, M> = Box<dyn Fn(&Tree, &Event) -> Option<M> + 'a>;

/// Navigation for a field holding `query`: Left at its start is `previous`, Right at its end `next`.
pub(crate) fn between_groups<'a, M: Clone + 'a>(
    query: &'a str,
    previous: Option<M>,
    next: Option<M>,
) -> Navigation<'a, M> {
    Box::new(move |tree, event| {
        if boundary_arrow(tree, event, query)? {
            next.clone()
        } else {
            previous.clone()
        }
    })
}

pub(crate) fn boundary_arrow(tree: &Tree, event: &Event, query: &str) -> Option<bool> {
    use iced::keyboard::{Event as KeyEvent, Key, key::Named};
    use iced::widget::text_input::{Value, cursor};
    let Event::Keyboard(KeyEvent::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };
    if !modifiers.is_empty() {
        return None;
    }
    let state = tree.state.downcast_ref::<InputState>();
    if !state.is_focused() {
        return None;
    }
    let value = Value::new(query);
    let cursor::State::Index(index) = state.cursor().state(&value) else {
        return None;
    };
    match key {
        Key::Named(Named::ArrowLeft) if index == 0 => Some(false),
        Key::Named(Named::ArrowRight) if index == value.len() => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_event(key: iced::keyboard::key::Named, modifiers: iced::keyboard::Modifiers) -> Event {
        use iced::keyboard::{Key, Location, key::Physical};
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(key),
            modified_key: Key::Named(key),
            physical_key: Physical::Unidentified(iced::keyboard::key::NativeCode::Unidentified),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }

    #[test]
    fn arrows_only_navigate_focused_unmodified_unselected_boundaries() {
        use iced::keyboard::{Modifiers, key::Named};
        let element: iced::Element<'_, (), crate::Theme> =
            iced::widget::text_input("", "a\u{301}b")
                .on_input(|_| ())
                .into();
        let mut input = Tree::new(&element);
        let input = &mut input;
        let left = key_event(Named::ArrowLeft, Modifiers::empty());
        let right = key_event(Named::ArrowRight, Modifiers::empty());
        let query = "a\u{301}b";
        assert_eq!(boundary_arrow(input, &left, query), None);
        input.state.downcast_mut::<InputState>().focus();
        assert_eq!(boundary_arrow(input, &right, query), Some(true));
        input
            .state
            .downcast_mut::<InputState>()
            .move_cursor_to_front();
        assert_eq!(boundary_arrow(input, &left, query), Some(false));
        assert_eq!(boundary_arrow(input, &right, query), None);
        for modifiers in [
            Modifiers::SHIFT,
            Modifiers::CTRL,
            Modifiers::ALT,
            Modifiers::LOGO,
        ] {
            assert_eq!(
                boundary_arrow(input, &key_event(Named::ArrowLeft, modifiers), query),
                None
            );
        }
        input.state.downcast_mut::<InputState>().move_cursor_to(1);
        assert_eq!(boundary_arrow(input, &left, query), None);
        assert_eq!(boundary_arrow(input, &right, query), None);
        input.state.downcast_mut::<InputState>().select_all();
        assert_eq!(boundary_arrow(input, &left, query), None);
        assert_eq!(boundary_arrow(input, &right, query), None);
        input
            .state
            .downcast_mut::<InputState>()
            .move_cursor_to_front();
        assert_eq!(boundary_arrow(input, &left, ""), Some(false));
        assert_eq!(boundary_arrow(input, &right, ""), Some(true));
    }
}
