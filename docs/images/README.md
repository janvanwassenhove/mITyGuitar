# Documentation Images

This directory contains screenshots and diagrams used in the mITyGuitar documentation.

## Image Guidelines

### Screenshots
- **Resolution**: 1920x1080 or higher
- **Format**: PNG with transparency where appropriate
- **Compression**: Optimize for web (keep under 500KB when possible)
- **Content**: Focus on relevant UI elements, crop unnecessary space

### Naming Convention
- `main-interface.png` - Main application interface
- `song-library.png` - Song library/management view
- `fretboard-view.png` - Live fretboard visualization
- `diagnostics-view.png` - Audio diagnostics panel
- `song-format-overview.png` - Song format diagram/example
- `controller-mapping.png` - Controller setup wizard
- `genre-selection.png` - Genre and pattern selection

## Planned Images

### User Interface Screenshots
- [ ] `main-interface.png` - Main app window with menu bar and controls
- [ ] `song-library.png` - Song library browser with upload/load/delete buttons
- [ ] `fretboard-view.png` - Live fretboard with pressed frets highlighted
- [ ] `diagnostics-view.png` - Audio performance diagnostics panel
- [ ] `chord-mapping-controls.png` - Genre selection and chord mapping settings

### Workflow Diagrams
- [ ] `song-format-overview.png` - Visual breakdown of `.mitychart.json` structure
- [ ] `architecture-diagram.png` - System architecture overview
- [ ] `audio-pipeline.png` - Audio processing flow diagram
- [ ] `controller-flow.png` - Input processing workflow

### Tutorial Screenshots
- [ ] `first-launch.png` - What users see when they first open the app
- [ ] `playing-chord.png` - Fretboard while playing a chord
- [ ] `loading-song.png` - Song loading process
- [ ] `genre-switching.png` - Changing between genres

## Usage in Documentation

Images are referenced using relative paths from each document:

**From root documents (README.md, QUICKSTART.md):**
```markdown
![Alt text](docs/images/image-name.png)
```

**From docs/ directory:**
```markdown
![Alt text](images/image-name.png)
```

### Image Optimization

Before adding images:
1. **Crop** to show only relevant content
2. **Optimize** file size without losing quality
3. **Add alt text** for accessibility
4. **Use descriptive captions** when helpful

### Example Usage

```markdown
![mITyGuitar Main Interface](images/main-interface.png)
*The main mITyGuitar interface showing fretboard, controls, and song information*
```

## Future Additions

As the application evolves, consider adding:
- **Feature demonstration GIFs** - Show chord playing in action
- **Comparison screenshots** - Before/after for new features
- **Mobile/responsive views** - If web version is developed
- **Dark/light theme examples** - UI variations

## Contributing Images

When contributing screenshots:
1. Use the latest development build
2. Show realistic usage scenarios
3. Include both empty states and active content
4. Ensure no sensitive information is visible
5. Follow the naming conventions above

---

*Images will be added as the application UI is finalized and screenshots become available.*