# Ghost in the Shell participant icons (local-only)

This folder is **intentionally not committed** (see repo root `.gitignore`).

You can optionally drop character portraits/icons here and Tachikoma will render them in the Think Tank participant picker.

## Why not committed?

The icon pack you linked is licensed as **“Free for personal desktop use only.”**  
Source: [Ghost in the Shell Icons Pack](https://www.iconarchive.com/show/ghost-in-the-shell-icons-by-iconfactory.html)

Because Tachikoma is public, we should **not** commit those assets into the repo.

## Expected filenames

Save icons with these names (PNG recommended):

- `motoko.png`
- `batou.png`
- `togusa.png`
- `aramaki.png`
- `ishikawa.png`
- `saito.png`
- `tachikoma.png`
- `laughing-man.png`
- `puppet-master.png`

## Avatar format used by the app

Participants use this format in code:

`asset:/gits-icons/<file>.png|<fallback-emoji>`

If the image is missing, the UI automatically falls back to the emoji.

