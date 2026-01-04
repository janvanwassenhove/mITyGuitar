# Song Format Guide

This guide explains how to create custom songs for mITyGuitar using the `.mitychart.json` format.

## Overview

mITyGuitar songs are JSON files that define:
- Song metadata (title, artist, links)
- Timing information (BPM, time signature, subdivision)
- Chord mappings to fret buttons
- Note charts with timing and chords
- Lyrics with word-level timing
- Song structure sections

## File Format: `.mitychart.json`
![Song Format Overview](images/song-format-overview.png)
*Overview of the mITyGuitar song format structure*
### Complete Example

```json
{
  "meta": {
    "title": "Example Song",
    "artist": "Artist Name",
    "youtube": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
    "spotify": "https://open.spotify.com/track/4iV5W9uYEdYUVa79Axb7Rh"
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
      "G": { "frets": ["YELLOW", "BLUE"] },
      "Am": { "frets": ["RED", "YELLOW"] },
      "F": { "frets": ["GREEN", "YELLOW", "BLUE"] }
    }
  },
  "lanes": [
    {
      "name": "Main",
      "events": [
        { "startBeat": 0, "dur": 4, "chord": "C" },
        { "startBeat": 4, "dur": 4, "chord": "G" },
        { "startBeat": 8, "dur": 4, "chord": "Am" },
        { "startBeat": 12, "dur": 4, "chord": "F" }
      ]
    }
  ],
  "lyrics": [
    { "startBeat": 0, "annotations": [
      { "word": "First", "timeBeat": "0.0" },
      { "word": "line", "timeBeat": "1.0" },
      { "word": "of", "timeBeat": "2.0" },
      { "word": "lyrics", "timeBeat": "3.0" }
    ]},
    { "startBeat": 4, "annotations": [
      { "word": "Second", "timeBeat": "4.0" },
      { "word": "line", "timeBeat": "5.0" },
      { "word": "here", "timeBeat": "6.0" }
    ]}
  ],
  "sections": [
    { "name": "Intro", "fromBeat": 0, "toBeat": 8 },
    { "name": "Verse", "fromBeat": 8, "toBeat": 24 },
    { "name": "Chorus", "fromBeat": 24, "toBeat": 40 }
  ]
}
```

## Section Reference

### 1. Meta (Metadata)
```json
"meta": {
  "title": "Song Title",           // Required: Display name
  "artist": "Artist Name",        // Required: Artist/band name
  "youtube": "https://...",       // Optional: YouTube link (shows ▶️ icon)
  "spotify": "https://...",       // Optional: Spotify link (shows 🎵 icon)
  "genre": "rock",                // Optional: Suggested genre
  "difficulty": "medium"          // Optional: Easy/Medium/Hard
}
```

### 2. Clock (Timing)
```json
"clock": {
  "bpm": 120,                     // Required: Beats per minute
  "timeSig": [4, 4],             // Required: Time signature [beats, note_value]
  "countInBars": 2,              // Optional: Count-in before song starts
  "subdivision": "8n"            // Required: Strumming pattern
}
```

**Subdivision Options:**
- `"4n"` (quarter notes) → 1 strum per beat
- `"8n"` (eighth notes) → 2 strums per beat
- `"16n"` (sixteenth notes) → 4 strums per beat

### 3. Mapping (Chord Definitions)
```json
"mapping": {
  "chords": {
    "C": { "frets": ["GREEN", "RED"] },
    "G7": { "frets": ["YELLOW", "BLUE", "ORANGE"] },
    "Em": { "frets": ["RED"] }
  }
}
```

**Available Frets:**
- `GREEN` (1st fret)
- `RED` (2nd fret) 
- `YELLOW` (3rd fret)
- `BLUE` (4th fret)
- `ORANGE` (5th fret)

![Fretboard Layout](images/fretboard-view.png)
*Live fretboard showing fret button colors and chord assignments*

**Chord Naming:**
- Use standard chord notation: `C`, `Am`, `F#dim`, `Gsus4`
- Keep names short for UI display
- Be consistent within a song

### 4. Lanes (Note Charts)
```json
"lanes": [
  {
    "name": "Main",               // Required: Lane identifier
    "events": [
      {
        "startBeat": 0,           // Required: When chord starts (beat number)
        "dur": 4,                 // Required: How long chord lasts (beats)
        "chord": "C"              // Required: Chord name (must exist in mapping)
      }
    ]
  }
]
```

**Multiple Lanes:**
```json
"lanes": [
  {
    "name": "Main",
    "events": [...]
  },
  {
    "name": "Solo",               // Solo fret buttons (Q-T keys)
    "events": [...]
  }
]
```

### 5. Lyrics (Optional)
```json
"lyrics": [
  {
    "startBeat": 0,               // Required: When this lyric line starts
    "annotations": [              // Required: Word-by-word timing
      {
        "word": "Hello",          // Required: The word to display
        "timeBeat": "0.0"         // Required: Exact timing (string format)
      },
      {
        "word": "world",
        "timeBeat": "1.0"
      }
    ]
  }
]
```

**Timing Format:**
- Use string format: `"0.0"`, `"1.5"`, `"2.25"`
- Decimal beats are supported for precise word timing
- Words should align with musical phrasing

### 6. Sections (Song Structure)
```json
"sections": [
  {
    "name": "Intro",              // Required: Section name for navigation
    "fromBeat": 0,                // Required: Starting beat
    "toBeat": 8                   // Required: Ending beat
  },
  {
    "name": "Verse 1",
    "fromBeat": 8,
    "toBeat": 24
  }
]
```

## Creating Songs

### Step 1: Plan Your Song
1. **Choose a song** with clear chord changes
2. **Find BPM** using a metronome or BPM detector
3. **Map out sections**: Intro, Verse, Chorus, Bridge, Outro
4. **Choose subdivision**: How fast should strumming be?

### Step 2: Define Chords
1. **Pick 3-5 chords** that work well together
2. **Assign to frets** based on musical relationship:
   - Green (1st) → Root/home chord
   - Red (2nd) → IV chord (subdominant)
   - Yellow (3rd) → V chord (dominant)  
   - Blue (4th) → Minor variations
   - Orange (5th) → Color/extension chords
3. **Test combinations** in the app to hear how they sound

### Step 3: Create the Chart
1. **Start with basic timing**: Use whole notes (4 beats per chord)
2. **Add chord changes** following the original song structure
3. **Refine timing** to match musical phrases
4. **Test frequently** in the app

### Step 4: Add Lyrics (Optional)
1. **Break into lines** that fit screen width (~6-8 words)
2. **Time each word** to match vocal melody
3. **Use decimal beats** for precise alignment
4. **Preview in app** to check readability

### Step 5: Test and Polish
1. **Play through completely** at least 3 times
2. **Check timing** - does it feel musical?
3. **Verify chord transitions** are smooth
4. **Get feedback** from other players

## Tips and Best Practices

### Musical Guidelines
- **Keep it simple**: 3-4 chords work better than 8-10
- **Logical progression**: I-vi-IV-V is always a winner
- **Match the original**: Stay true to the song's feel
- **Consider difficulty**: New players prefer simpler charts

### Technical Tips
- **Beat numbering starts at 0** (not 1)
- **Duration is in beats**, not measures
- **Chord changes should align** with strong beats when possible
- **Test different subdivisions** to find the right strumming feel

### Common Patterns
**Pop/Rock (4/4 time):**
- I-V-vi-IV progression
- 8th note subdivision
- 4-beat chord durations

**Folk/Acoustic:**
- I-IV-V-I progression  
- Quarter note subdivision
- Longer chord durations (8+ beats)

**Punk/Fast:**
- Power chords only
- 16th note subdivision
- Quick changes (1-2 beats)

## File Management

### Organization
```
assets/songs/
├── README.md                    # This file
├── simple-blues.mitychart.json  # Included examples
├── greensleeves.mitychart.json
└── your-song.mitychart.json     # Your creations
```

### Testing Songs
1. **Load in app**: Song Play → 📁 Song Library → ▶ Load
2. **Practice mode**: Play at slower speed to check timing
3. **Check lyrics**: Do words appear at right time?
4. **Verify chords**: Do fret combinations feel good?

### Sharing Songs
- Files can be shared directly (just JSON)
- Include original song reference for others
- Consider adding difficulty rating
- Test on different skill levels

## Troubleshooting

### Song Won't Load
- **Check JSON syntax**: Use online validator
- **Verify required fields**: meta.title, meta.artist, clock.bpm, etc.
- **Check chord references**: All chords in lanes must exist in mapping

### Timing Issues
- **Double-check BPM**: Use metronome to verify
- **Verify subdivision**: Does strumming pattern match?
- **Check beat alignment**: Major chord changes on beat 0, 4, 8, etc.

### Gameplay Problems
- **Too fast**: Reduce BPM or use longer chord durations
- **Too hard**: Simplify chord progressions or use fewer frets
- **Awkward transitions**: Rearrange fret assignments for smoother finger movement

## Examples

See included songs for reference:
- **[simple-blues.mitychart.json](../assets/songs/simple-blues.mitychart.json)** - Basic 12-bar blues
- **[greensleeves.mitychart.json](../assets/songs/greensleeves.mitychart.json)** - Traditional folk song
- **[Djo - End Of Beginning.json](../assets/songs/Djo%20-%20End%20Of%20Beginning.json)** - Modern pop example

Happy charting! 🎸🎵