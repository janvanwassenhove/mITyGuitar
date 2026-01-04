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
      { "word": "First", "timeBeat": 0.0 },
      { "word": "line", "timeBeat": 0.5 },
      { "word": "of", "timeBeat": 0.75 },
      { "word": "lyrics", "timeBeat": 1.0 }
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

### Timing and Beat Format

**All timing values in `.mitychart.json` use QUARTER-NOTE BEATS as the canonical unit.**

#### Time Signature Math

```javascript
// Calculate quarter-note beats per bar from time signature
beatsPerBar = timeSig[0] * (4 / timeSig[1])

// Examples:
// 4/4 time: 4 * (4/4) = 4 quarter-note beats per bar
// 3/4 time: 3 * (4/4) = 3 quarter-note beats per bar
// 6/8 time: 6 * (4/8) = 3 quarter-note beats per bar

// Convert beats to seconds
secondsPerBeat = 60 / bpm
timeInSeconds = beatValue * secondsPerBeat
```

#### Lyric Timing Format

Each word in `lyrics[].annotations[]` has a `timeBeat` field that represents **quarter-note beats relative to the lyric line's `startBeat`**.

**Correct format (quarter-note beats):**
```json
{
  "startBeat": 4,
  "annotations": [
    { "word": "First", "timeBeat": 0.0 },
    { "word": "word", "timeBeat": 1.0 },
    { "word": "second", "timeBeat": 2.0 },
    { "word": "bar", "timeBeat": 4.0 }
  ]
}
```

**In 4/4 time:** `timeBeat: 4.0` means "one full bar (4 beats) after startBeat"  
**In 6/8 time:** `timeBeat: 3.0` means "one full bar (3 quarter-note beats) after startBeat"

**❌ Invalid format (bar fractions - will be auto-migrated):**
```json
// DON'T USE: These values will be detected as invalid and auto-converted
{ "word": "First", "timeBeat": 0.0 },
{ "word": "word", "timeBeat": 0.33 },  // Wrong! This is a bar fraction
{ "word": "second", "timeBeat": 0.67 }, // Wrong! Should be 2.67 in 4/4
{ "word": "bar", "timeBeat": 1.0 }      // Would mean 1 beat, not 1 bar
```

The app will automatically detect and migrate invalid bar-fraction data, but new charts should use quarter-note beats from the start.

## Creating Custom Songs

You can create your own song charts by following the format above. Make sure to:

- Use valid chord names in the mapping
- Assign frets: GREEN, RED, YELLOW, BLUE, ORANGE
- Set appropriate BPM and time signature
- Set subdivision for strumming pattern (8n, 16n, or 4n)
- Use `startBeat` for event timing in lanes (quarter-note beats)
- **Add word-level lyrics with `annotations` array:**
  - Each annotation has `word` (string) and `timeBeat` (number)
  - `timeBeat` is measured in **quarter-note beats relative to the line's `startBeat`**
  - Example: In 4/4 time, `timeBeat: 4.0` means "one bar after startBeat"
  - Do NOT use bar fractions (0.33, 0.67) - these are invalid and will be auto-migrated
- Optionally add YouTube and Spotify links for reference (displayed as icons in the play view)
- Test in-game to ensure proper timing

**Timing Formula:**
```
absoluteBeat = lyrics[i].startBeat + annotations[j].timeBeat
```

Happy playing! 🎸
