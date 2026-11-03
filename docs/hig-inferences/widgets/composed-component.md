# ComposedComponent

No libadwaita or GTK counterpart. The opposite of `MetaComponent`: a meta component has one piece of content and chooses how to present it; a composed component has **several components and one shared state**, and renders as a single widget.

## What it is

Components that edit one value, shown together: a histogram over a range slider, a color box next to a hex entry, a calendar beside a date entry. Whichever part the user touches, the application receives one value through one message, and every part is built from that value on the next view, so the parts cannot disagree.

## Anatomy and behavior

- **State.** The app owns it. `ComposedComponent::new(state, on_change)`.
- **Parts.** `.part(|state| element)` builds a part from the current state. The element's messages are the *next state*. Parts are laid out in order, in a column (`ComposeAxis::Vertical`, default) or a row.
- **Rule.** `.normalize(|next| next)` runs on every next state before it is reported, so an invariant (ordering, clamping) lives in one place, not in each part.

## iced mapping

Each part's element is mapped through the rule and `on_change`. It is a plain column or row of those elements, so focus, overlays and keyboard handling are the parts' own.
