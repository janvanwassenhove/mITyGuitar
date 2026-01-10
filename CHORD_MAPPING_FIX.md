# Chord Mapping Error Fix Guide

## Problem: "Chord 'F' not found in mapping"

This error occurs when a song file references a chord in its events that isn't defined in the mapping section.

## How to Fix:

### Option 1: Add Missing Chord to Mapping
If the chord should be playable, add it to the mapping section:

```json
{
  "mapping": {
    "chords": {
      "F": {
        "frets": ["RED", "YELLOW"]
      }
      // ... other chords
    }
  }
}
```

### Option 2: Change Chord Reference 
If the chord was misnamed, update the event to use the correct chord name:

```json
{
  "lanes": [
    {
      "name": "Main", 
      "events": [
        {
          "startBeat": 4,
          "dur": 4,
          "chord": "Fadd9"  // Changed from "F" to "Fadd9"
        }
      ]
    }
  ]
}
```

## Common Fret Color Mappings:

- **F Major**: `["RED", "YELLOW"]`
- **F Minor**: `["RED", "ORANGE"]`  
- **Fadd9**: `["GREEN", "RED"]`
- **Fmaj7**: `["RED", "YELLOW", "BLUE"]`

## Validation Tool:
Run `node validate-songs.js` to check all song files for mapping issues.

## Prevention:
1. Always define chords in mapping before using them in events
2. Use consistent chord naming (case-sensitive)
3. Validate songs after editing
4. Check for typos in chord names