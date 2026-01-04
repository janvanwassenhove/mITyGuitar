# Icon Placeholder Note

The Tauri configuration references icon files in `src-tauri/icons/`:
- 32x32.png
- 128x128.png
- 128x128@2x.png
- icon.icns (macOS)
- icon.ico (Windows)

## To Add Icons

1. Create a simple guitar icon image (PNG format)
2. Use an icon generator to create all required sizes:
   - https://icon.kitchen/
   - https://www.iconsgenerator.com/
3. Place generated icons in `apps/desktop/src-tauri/icons/`

## Temporary Solution

For development, you can:
1. Copy any PNG file and rename it to the required sizes
2. Tauri will use these placeholders until proper icons are added
3. The app will still work without icons (just missing taskbar/app icon)

## Required Structure

```
apps/desktop/src-tauri/icons/
├── 32x32.png
├── 128x128.png
├── 128x128@2x.png
├── icon.icns
└── icon.ico
```

Icons are bundled automatically during `tauri build`.
