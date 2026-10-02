# Renderer patch

Source: pop-os/iced at 10db38f982001a714bd94e99a082368762b378ee, the iced submodule of the locked libcosmic 3b8ad45950f5d23c8550e18e628f6e70b7089d89. MIT license and attribution are preserved. The standalone manifest resolves workspace dependencies against the same locked sources.

One functional change in src/image/mod.rs: prepare every requested image batch even when all uploads fail. This clears stale instance counts and advances the prepared layer index, preventing artwork or sidebar icons from previous frames appearing after resize or navigation. No renderer API is changed.

The X11 smoke test exercises a populated catalog, navigation, empty search and narrow-window resize. Inspect its screenshots when updating or removing this patch.
