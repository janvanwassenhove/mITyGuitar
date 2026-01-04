# MITyGuitar Song Library

This directory contains song charts for MITyGuitar in `.mitychart.json` format.

## Using the Song Library

1. **Upload Songs**: Click the "⬆ Upload Song" button in the Song Play view
2. **Browse Library**: Click "📁 Song Library" to see all available songs
3. **Load Songs**: Click "▶ Load" next to any song to play it
4. **Delete Songs**: Click the 🗑 icon to remove a song from your library

## Song Format

Songs are stored in JSON format with the `.mitychart.json` extension. Example structure:

```json
{
  "meta": {
    "title": "Song Title",
    "artist": "Artist Name",
    "youtube": "https://www.youtube.com/watch?v=...",
    "spotify": "https://open.spotify.com/track/..."
  },
  "clock": {
    "bpm": 120,
    "timeSig": [4, 4],
    "countInBars": 2,
    "subdivision": "8n"
  },
  "mapping": {
    "chords": {
      "C": { "frets": ["GREEN", "RED"] },
      "G": { "frets": ["YELLOW", "BLUE"] }
    }
  },
  "lanes": [
    {
      "name": "Main",
      "events": [
        { "startBeat": 0, "dur": 1, "chord": "C" },
        { "startBeat": 2, "dur": 1, "chord": "G" }
      ]
    }
  ],
  "lyrics": [
    { "startBeat": 0, "annotations": [
      { "word": "First", "timeBeat": "0.0" },
      { "word": "line", "timeBeat": "0.5" },
      { "word": "of", "timeBeat": "0.75" },
      { "word": "lyrics", "timeBeat": "1.0" }
    ]}
  ],
  "sections": [
    { "name": "Intro", "fromBeat": 0, "toBeat": 8 },
    { "name": "Verse", "fromBeat": 8, "toBeat": 24 }
  ]
}
```

### Subdivision Field

The `subdivision` field in the `clock` object controls strumming patterns:
- `"8n"` (eighth notes) → 2 strums per beat
- `"16n"` (sixteenth notes) → 4 strums per beat
- `"4n"` (quarter notes) → 1 strum per beat (default)

## Creating Custom Songs

You can create your own song charts by following the format above. Make sure to:

- Use valid chord names in the mapping
- Assign frets: GREEN, RED, YELLOW, BLUE, ORANGE
- Set appropriate BPM and time signature
- Set subdivision for strumming pattern (8n, 16n, or 4n)
- Use `startBeat` for event timing in lanes
- Add word-level lyrics with `annotations` array containing `word` and `timeBeat` for each word
- Optionally add YouTube and Spotify links for reference (displayed as icons in the play view)
- Test in-game to ensure proper timing

Happy playing! 🎸
