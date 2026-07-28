# FGIT Blueprint References

These are the two screenshots pasted by the user into the FGIT project. They
are product references, not decorative inspiration.

## `FGIT-TUI-UI-UX-blueprint.png`

This image defines the TUI workflow. It contains the small state, big state,
message input, destination choice, `Fucking push` confirmation, per-file
progress, commit number result, navigation question, and responsive layout
idea. The full-screen TUI must preserve these relationships while correcting
the local-commit versus remote-push distinction.

## `FGIT-CLI-explanation.png`

This image defines the CLI workflow. It contains the `fgit c` automatic path,
preview path, file-selection path, destination-selection path, combined flags,
custom-message path, and the transition from CLI command to visual result.

The documented command forms normalize the sketch into safe syntax:

```text
fgit                 full TUI
fgit c               automatic commit path
fgit com             alias for automatic commit path
fgit commit          alias for automatic commit path
fgit c -p            preview automatic commit
fgit c -s            select files before automatic commit
fgit c -r            choose post-commit destination
fgit c -s -r         select files and destination
fgit c -m "message"  use a custom message
```

The command parser must treat the custom message as data, not shell code.
