# Versioned desktop configuration history

The toolkit is statically linked. Applications built against different toolkit
versions can read the same `desktop.conf` without coordination.

- The bundled read-only registry in `src/config_history.rs` is the source of
  truth for property names and their lifecycle metadata.
- `since` records introduction; `deprecated` records the **first** toolkit
  version in which referencing the property in a new build is forbidden.
- Historical properties remain in the registry permanently, and older
  applications retain their own parsers.
- Unknown properties are ignored by application configuration resolution.
  Applications must never remove or rewrite unknown user settings.
- `replacement` is informational, not an automatic alias or migration.
  Different toolkit generations may use independent property values.
- New build tooling should call `validate_build_properties` for properties
  explicitly referenced by build artifacts. Existing user desktop.conf files
  are runtime input, **not** build artifacts and must not fail a build.
- The registry currently starts with the existing 0.1.0 keys. No properties
  are deprecated yet.

A future Tweaks UI can show all active properties under **Latest** and
historical entries under **Deprecated**, with small version metadata.

This change intentionally does not add platform adapters, migration, Tweaks,
or a new configuration file format.
