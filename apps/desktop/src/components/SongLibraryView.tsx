import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openUrl } from "@tauri-apps/plugin-shell";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import SongUploadDialog from "./SongUploadDialog";
import "./SongLibraryView.css";

interface SongLibraryEntry {
  id: string;
  title: string;
  artist: string;
  filename: string;
}

interface SongChart {
  meta: {
    title: string;
    artist: string;
    youtube?: string;
    spotify?: string;
  };
  clock: {
    bpm: number;
    timeSig: [number, number];
    countInBars?: number;
    key?: string;
    subdivision?: string; // e.g., "8n", "16n"
  };
  mapping: {
    chords: Record<string, { frets: string[] }>;
  };
  lanes: Array<{
    name: string;
    events: ChordEvent[];
  }>;
  lyrics?: LyricEvent[];
}

interface ChordEvent {
  startBeat: number;
  dur: number;
  chord: string;
}

interface LyricEvent {
  startBeat: number;
  text?: string; // Legacy format
  annotations?: Array<{ 
    word: string; 
    timeBeat: string | number; // SPEC: Quarter-note beat offset from startBeat (parse as number)
    // Note: Bar-fraction values (0.33, 0.67, etc.) are INVALID and will be auto-migrated
  }>; // New format
}

interface LyricLine {
  lyrics: string;
  words: Array<{ word: string; position: number }>; // Individual word positions
  startBeat: number;
  endBeat: number;
  chords: Array<{ position: number; chord: string }>;
  lineMarkers: Array<{ beat: number; label: string; position: number }>;
}

export default function SongLibraryView() {
  const [songLibrary, setSongLibrary] = useState<SongLibraryEntry[]>([]);
  const [selectedSong, setSelectedSong] = useState<SongChart | null>(null);
  const [selectedSongFilename, setSelectedSongFilename] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [timelineMode, setTimelineMode] = useState<'beats' | 'seconds'>('beats');
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [uploadResult, setUploadResult] = useState<{ songName: string; isError: boolean; errorMessage?: string } | null>(null);
  const [editingTimeSig, setEditingTimeSig] = useState(false);
  const [timeSigNumerator, setTimeSigNumerator] = useState<string>('4');
  const [timeSigDenominator, setTimeSigDenominator] = useState<string>('4');

  useEffect(() => {
    loadSongLibrary();
  }, []);

  const loadSongLibrary = async () => {
    try {
      const library = await invoke<SongLibraryEntry[]>("song_list_library");
      setSongLibrary(library);
    } catch (err) {
      console.error("Failed to load song library:", err);
      setError("Failed to load song library");
    }
  };

  const handleStartEditTimeSig = () => {
    if (selectedSong) {
      setTimeSigNumerator(selectedSong.clock.timeSig[0].toString());
      setTimeSigDenominator(selectedSong.clock.timeSig[1].toString());
      setEditingTimeSig(true);
    }
  };

  const handleSaveTimeSig = async () => {
    if (!selectedSong || !selectedSongFilename) return;
    
    const numerator = parseInt(timeSigNumerator);
    const denominator = parseInt(timeSigDenominator);
    
    // Validate input
    if (isNaN(numerator) || isNaN(denominator) || numerator < 1 || denominator < 1) {
      alert('Invalid time signature. Please enter positive numbers.');
      return;
    }
    
    // Validate common denominators (1, 2, 4, 8, 16)
    if (![1, 2, 4, 8, 16].includes(denominator)) {
      if (!confirm(`${denominator} is an unusual denominator. Continue anyway?`)) {
        return;
      }
    }
    
    try {
      // Update the song object
      const updatedSong = {
        ...selectedSong,
        clock: {
          ...selectedSong.clock,
          timeSig: [numerator, denominator] as [number, number]
        }
      };
      
      // Save using backend command (reuse song_save_to_library)
      await invoke('song_save_to_library', {
        json: JSON.stringify(updatedSong, null, 2),
        filename: selectedSongFilename
      });
      
      // Update local state
      setSelectedSong(updatedSong);
      setEditingTimeSig(false);
      
      console.log(`Time signature updated to ${numerator}/${denominator}`);
    } catch (err) {
      console.error('Failed to save time signature:', err);
      alert(`Failed to save time signature changes: ${err}`);
    }
  };

  const handleCancelEditTimeSig = () => {
    setEditingTimeSig(false);
  };

  const handleUploadSong = async () => {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{
          name: "MITyGuitar Chart",
          extensions: ["json", "mitychart.json"]
        }]
      });

      if (!selected) return;

      const path = typeof selected === "string" ? selected : (selected as any).path;
      
      // Read file using Tauri fs plugin
      const json = await readTextFile(path);
      
      // Get filename from path
      const filename = path.split(/[\\\/]/).pop() || "song.mitychart.json";
      
      // Save to library
      const savedFilename = await invoke<string>("song_save_to_library", { json, filename });
      
      // Reload library
      await loadSongLibrary();
      
      // Show success dialog
      setUploadResult({ songName: savedFilename, isError: false });
      setShowUploadDialog(true);
    } catch (err) {
      // Show error dialog
      setUploadResult({ 
        songName: "", 
        isError: true, 
        errorMessage: String(err)
      });
      setShowUploadDialog(true);
    }
  };

  const handleSelectSong = async (filename: string) => {
    try {
      setLoading(true);
      setError(null);
      setSelectedSongFilename(filename);

      // Check if it's a default song
      const isDefaultSong =
        filename === "greensleeves.mitychart.json" ||
        filename === "simple-blues.mitychart.json";

      // Load the song into the engine first
      if (isDefaultSong) {
        if (filename === "greensleeves.mitychart.json") {
          await invoke("song_load_default_chart");
        } else {
          await invoke("song_load_chart_from_path", { path: `assets/songs/${filename}` });
        }
      } else {
        await invoke("song_load_from_library", { filename });
      }

      // Now get the chart JSON from the engine
      const chartJson = await invoke<string>("song_get_chart");
      if (!chartJson) {
        throw new Error("Failed to get chart from engine");
      }

      const chart = JSON.parse(chartJson) as SongChart;
      setSelectedSong(chart);
    } catch (err) {
      console.error("Failed to load song:", err);
      setError(`Failed to load song: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteSong = async (filename: string, title: string) => {
    // Prevent deleting default songs
    if (filename === "greensleeves.mitychart.json" || filename === "simple-blues.mitychart.json") {
      alert("Cannot delete default songs");
      return;
    }
    
    if (!confirm(`Delete "${title}"?`)) return;
    
    try {
      await invoke("song_delete_from_library", { filename });
      // If the deleted song was selected, clear it
      if (selectedSong && selectedSong.meta.title === title) {
        setSelectedSong(null);
      }
      await loadSongLibrary();
    } catch (err) {
      alert(`Failed to delete song: ${err}`);
    }
  };

  const openExternalLink = (url: string) => {
    openUrl(url).catch((err) => console.error("Failed to open URL:", err));
  };

  const isDefaultSong = (filename: string): boolean => {
    return filename === "greensleeves.mitychart.json" || filename === "simple-blues.mitychart.json";
  };

  const getUniqueChords = (chart: SongChart): string[] => {
    const chords = new Set<string>();
    chart.lanes.forEach((lane) => {
      lane.events.forEach((event) => {
        chords.add(event.chord);
      });
    });
    return Array.from(chords).sort();
  };

  // Time signature utility functions
  const beatsPerBarQN = (timeSig: [number, number]): number => {
    // Number of quarter-note beats per bar: top * (4 / bottom)
    // 4/4 => 4 * (4/4) = 4, 6/8 => 6 * (4/8) = 3, 3/4 => 3 * (4/4) = 3
    return timeSig[0] * (4 / timeSig[1]);
  };

  const beatToSecondsQN = (beatQN: number, bpm: number): number => {
    // Convert quarter-note beats to seconds
    return beatQN * (60 / bpm);
  };

  // @ts-ignore - Utility function for future use
  const secondsToBeatQN = (seconds: number, bpm: number): number => {
    // Convert seconds to quarter-note beats
    return seconds * bpm / 60;
  };

  // @ts-ignore - Utility function for future use
  const beatQNToBarBeat = (beatQN: number, timeSig: [number, number]): { bar: number; beat: number } => {
    // Convert absolute quarter-note beat to bar/beat position (1-indexed for display)
    const beatsPerBar = beatsPerBarQN(timeSig);
    const bar = Math.floor(beatQN / beatsPerBar) + 1; // Bars are 1-indexed
    const beat = (beatQN % beatsPerBar) + 1; // Beats within bar are 1-indexed
    return { bar, beat };
  };

  // @ts-ignore - Legacy function kept for compatibility
  const beatsToSeconds = (beats: number, bpm: number, countInBars: number = 0, timeSig: [number, number] = [4, 4]): number => {
    // Legacy function - keeping for compatibility
    const countInBeats = countInBars * timeSig[0];
    const totalBeats = beats + countInBeats;
    return (totalBeats / bpm) * 60;
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getTimeLabel = (beat: number, bpm: number, timeSig?: [number, number]): string => {
    if (timelineMode === 'beats') {
      if (timeSig) {
        const { bar, beat: beatNum } = beatQNToBarBeat(beat, timeSig);
        return `${bar}.${beatNum}`;
      }
      return `${Math.round(beat)}`;
    } else {
      // Use quarter-note beat to seconds conversion
      const seconds = beatToSecondsQN(beat, bpm);
      return formatTime(seconds);
    }
  };

  // VALIDATION: Detect INVALID bar-fraction data (should be quarter-note beats per spec)
  const detectInvalidBarFractionData = (lyric: LyricEvent): boolean => {
    if (!lyric.annotations || lyric.annotations.length < 3) return false;
    
    // Parse timeBeat values as numbers (per spec)
    const timeBeatValues = lyric.annotations.map(ann => 
      typeof ann.timeBeat === 'number' ? ann.timeBeat : parseFloat(ann.timeBeat)
    );
    
    // Check for common bar fraction patterns (0.33, 0.67, 0.25, 0.5, 0.75)
    // These indicate INVALID data that used bar fractions instead of quarter-note beats
    const hasCommonFractions = timeBeatValues.some(val => {
      const remainder = val % 1;
      return Math.abs(remainder - 0.33) < 0.05 || 
             Math.abs(remainder - 0.67) < 0.05 || 
             Math.abs(remainder - 0.25) < 0.05 || 
             Math.abs(remainder - 0.5) < 0.05 || 
             Math.abs(remainder - 0.75) < 0.05;
    });
    
    if (hasCommonFractions) return true;
    
    // Fallback: Calculate span in quarter-note beats if interpreted as beats
    const spanBeatsQN = Math.max(...timeBeatValues) - Math.min(...timeBeatValues);
    
    // If words >= 5 and span < 2 quarter beats, likely invalid bar fractions
    return lyric.annotations.length >= 5 && spanBeatsQN < 2.0;
  };

  // AUTO-MIGRATION: Convert INVALID bar-fraction data to proper quarter-note beats
  const convertBarFractionToBeats = (timeBeat: string | number, timeSig: [number, number]): number => {
    const timeValue = typeof timeBeat === 'number' ? timeBeat : parseFloat(timeBeat);
    const beatsPerBar = beatsPerBarQN(timeSig);
    
    // Convert bar fraction to quarter-note beats (per spec)
    return timeValue * beatsPerBar;
  };

  // VALIDATION: Check timing data against spec (quarter-note beats)
  const validateLyricTiming = (lyric: LyricEvent, timeSig: [number, number]): string[] => {
    const warnings: string[] = [];
    
    if (!lyric.annotations || lyric.annotations.length === 0) return warnings;
    
    const wordCount = lyric.annotations.length;
    // Parse timeBeat as numbers per spec
    const timeBeatValues = lyric.annotations.map(ann => 
      typeof ann.timeBeat === 'number' ? ann.timeBeat : parseFloat(ann.timeBeat)
    );
    const spanBeatsQN = Math.max(...timeBeatValues) - Math.min(...timeBeatValues);
    
    const beatsPerBar = beatsPerBarQN(timeSig);
    const minExpectedSpan = Math.min(2.0, 0.5 * beatsPerBar);
    
    if (wordCount >= 5 && spanBeatsQN < minExpectedSpan) {
      warnings.push(`Dense timing: ${wordCount} words in ${spanBeatsQN.toFixed(1)} quarter-note beats (expected ≥${minExpectedSpan.toFixed(1)})`);
    }
    
    return warnings;
  };

  const getLineTimelineMarkers = (startBeat: number, endBeat: number, bpm: number, timeSig: [number, number]) => {
    const markers: Array<{ beat: number; label: string; position: number }> = [];
    const lineRange = endBeat - startBeat;
    const interval = timelineMode === 'beats' ? 1 : 2; // Show every beat or every 2 beats for seconds
    
    for (let beat = Math.ceil(startBeat / interval) * interval; beat <= endBeat; beat += interval) {
      if (beat >= startBeat && beat <= endBeat) {
        const position = ((beat - startBeat) / lineRange) * 100;
        markers.push({ 
          beat, 
          label: getTimeLabel(beat, bpm, timeSig),
          position: Math.max(0, Math.min(100, position))
        });
      }
    }
    
    return markers;
  };

  // @ts-ignore - Function for future timeline features
  const getFullTimelineMarkers = (chart: SongChart) => {
    if (!chart.lyrics || chart.lyrics.length === 0) return { markers: [], maxBeat: 0 };
    
    const minBeat = Math.min(...chart.lyrics.map(l => l.startBeat ?? (l as any).beat ?? 0));
    const maxBeat = Math.max(...chart.lyrics.map(l => l.startBeat ?? (l as any).beat ?? 0));
    const markers: Array<{ beat: number; label: string; position: number }> = [];
    const interval = 8; // Mark every 8 beats
    const totalRange = maxBeat - minBeat;
    
    for (let beat = Math.floor(minBeat / interval) * interval; beat <= maxBeat; beat += interval) {
      const position = ((beat - minBeat) / totalRange) * 100;
      markers.push({ 
        beat, 
        label: getTimeLabel(beat, chart.clock.bpm),
        position
      });
    }
    
    return { markers, minBeat, maxBeat };
  };

  const formatLyricsWithChords = (chart: SongChart): LyricLine[] => {
    if (!chart.lyrics || chart.lyrics.length === 0) {
      return [];
    }

    // Get all chord events from all lanes
    const allChordEvents: ChordEvent[] = [];
    chart.lanes.forEach((lane) => {
      allChordEvents.push(...lane.events);
    });

    // Sort both arrays by beat (support both old and new formats)
    const sortedChords = [...allChordEvents].sort((a, b) => {
      const beatA = a.startBeat ?? (a as any).beat ?? 0;
      const beatB = b.startBeat ?? (b as any).beat ?? 0;
      return beatA - beatB;
    });
    const sortedLyrics = [...chart.lyrics].sort((a, b) => {
      const beatA = a.startBeat ?? (a as any).beat ?? 0;
      const beatB = b.startBeat ?? (b as any).beat ?? 0;
      return beatA - beatB;
    });

    // Calculate beat range for timeline
    const maxBeat = Math.max(...sortedLyrics.map(l => l.startBeat ?? (l as any).beat ?? 0), ...allChordEvents.map(c => c.startBeat ?? (c as any).beat ?? 0)) + 16; // Extend beyond last event

    const lines: LyricLine[] = [];
    
    // Find the first lyric beat
    const firstLyricBeat = sortedLyrics.length > 0 ? (sortedLyrics[0].startBeat ?? (sortedLyrics[0] as any).beat ?? 0) : 0;
    
    // Create sections from beat 0 to cover the entire timeline
    const sectionSize = 4; // 4 beats per section
    
    // Add empty sections from 0 to first lyric
    for (let beat = 0; beat < firstLyricBeat; beat += sectionSize) {
      const sectionEnd = Math.min(beat + sectionSize, firstLyricBeat);
      
      // Find chords in this empty section
      const chordsInRange = sortedChords.filter(c => {
        const chordBeat = c.startBeat ?? (c as any).beat ?? 0;
        return chordBeat >= beat && chordBeat < sectionEnd;
      }).map(c => {
        const chordBeat = c.startBeat ?? (c as any).beat ?? 0;
        const position = ((chordBeat - beat) / (sectionEnd - beat)) * 100;
        return { chord: c.chord, position: Math.max(0, Math.min(100, position)) };
      });
      
      const lineMarkers = getLineTimelineMarkers(beat, sectionEnd, chart.clock.bpm, chart.clock.timeSig);
      
      lines.push({
        startBeat: beat,
        endBeat: sectionEnd,
        lyrics: '', // Empty section
        words: [],
        lineMarkers,
        chords: chordsInRange
      });
    }

    // Process actual lyrics
    sortedLyrics.forEach((lyric, lyricIndex) => {
      const lyricBeat = lyric.startBeat ?? (lyric as any).beat ?? 0;
      
      // Determine the end beat for this line
      let lineEndBeat: number;
      if (lyric.annotations && lyric.annotations.length > 0) {
        // VALIDATION: Check if data contains invalid bar fractions
        const hasInvalidData = detectInvalidBarFractionData(lyric);
        
        // Get the last annotation's timing
        const lastAnnotation = lyric.annotations[lyric.annotations.length - 1];
        let lastWordBeatOffset: number;
        
        if (hasInvalidData) {
          // AUTO-MIGRATE: Convert invalid bar-fraction data to proper quarter-note beats
          lastWordBeatOffset = convertBarFractionToBeats(lastAnnotation.timeBeat, chart.clock.timeSig);
          console.warn(`Line ${lyricIndex}: INVALID bar-fraction data detected, auto-migrating: ${lastAnnotation.timeBeat} bars → ${lastWordBeatOffset} beats`);
        } else {
          // Parse as quarter-note beat offset (per spec)
          lastWordBeatOffset = typeof lastAnnotation.timeBeat === 'number' ? 
            lastAnnotation.timeBeat : parseFloat(lastAnnotation.timeBeat);
        }
        
        lineEndBeat = lyricBeat + lastWordBeatOffset + 0.5; // Add buffer
        console.log(`Line ${lyricIndex}: lastAnnotation.timeBeat = ${lastAnnotation.timeBeat}, offset = ${lastWordBeatOffset} beats, lineEndBeat = ${lineEndBeat}`);
      } else {
        // Fallback to next lyric's start or max beat
        lineEndBeat = lyricIndex < sortedLyrics.length - 1 
          ? (sortedLyrics[lyricIndex + 1].startBeat ?? (sortedLyrics[lyricIndex + 1] as any).beat ?? maxBeat + 8)
          : maxBeat + 8;
        console.log(`Line ${lyricIndex}: fallback lineEndBeat = ${lineEndBeat}`);
      }

      // Process words with individual positions using quarter-note beats (per spec)
      const words: Array<{ word: string; position: number }> = [];
      let lyricText = '';
      const warnings: string[] = [];
      
      if (lyric.annotations && lyric.annotations.length > 0) {
        // VALIDATION: Check for invalid bar-fraction data
        const hasInvalidData = detectInvalidBarFractionData(lyric);
        
        if (hasInvalidData) {
          warnings.push('INVALID DATA: timeBeat values appear to be bar fractions (should be quarter-note beats per spec)');
        }
        
        // Validate timing
        const timingWarnings = validateLyricTiming(lyric, chart.clock.timeSig);
        warnings.push(...timingWarnings);
        
        // Calculate word positions
        const lineRange = lineEndBeat - lyricBeat;
        lyric.annotations.forEach(ann => {
          let wordBeatQN: number;
          
          if (hasInvalidData) {
            // AUTO-MIGRATE: Convert invalid bar-fraction data to proper quarter-note beats
            wordBeatQN = convertBarFractionToBeats(ann.timeBeat, chart.clock.timeSig);
          } else {
            // Parse timeBeat as quarter-note beat offset from startBeat (per spec)
            wordBeatQN = typeof ann.timeBeat === 'number' ? ann.timeBeat : parseFloat(ann.timeBeat);
          }
          
          const position = lineRange > 0 ? (wordBeatQN / lineRange) * 100 : 0;
          words.push({
            word: ann.word,
            position: Math.max(0, Math.min(95, position))
          });
        });
        
        lyricText = lyric.annotations.map(ann => ann.word).join(' ');
        
        // Log warnings
        if (warnings.length > 0) {
          console.warn(`Line ${lyricIndex} validation:`, warnings);
        }
      } else {
        // Legacy format without annotations
        lyricText = lyric.text || '';
        words.push({ word: lyricText, position: 0 });
      }

      // Find chords in this line's time range
      const chordsInRange = sortedChords.filter(c => {
        const chordBeat = c.startBeat ?? (c as any).beat ?? 0;
        return chordBeat >= lyricBeat && chordBeat < lineEndBeat;
      }).map(c => {
        const chordBeat = c.startBeat ?? (c as any).beat ?? 0;
        const position = ((chordBeat - lyricBeat) / (lineEndBeat - lyricBeat)) * 100;
        return { chord: c.chord, position: Math.max(0, Math.min(100, position)) };
      });

      // Generate timeline markers for this line
      const lineMarkers = getLineTimelineMarkers(lyricBeat, lineEndBeat, chart.clock.bpm, chart.clock.timeSig);

      console.log(`Line ${lyricIndex}: beat ${lyricBeat}-${lineEndBeat}, markers: ${lineMarkers.length}, chords: ${chordsInRange.length}`);

      lines.push({
        startBeat: lyricBeat,
        endBeat: lineEndBeat,
        lyrics: lyricText,
        words,
        lineMarkers,
        chords: chordsInRange
      });
    });

    return lines;
  };

  return (
    <div className="song-library-view">
      <div className="library-container">
        <div className="library-sidebar">
          <div className="library-header">
            <h2>Song Library</h2>
            <span className="song-count">{songLibrary.length} songs</span>
            <button className="upload-button" onClick={handleUploadSong} title="Upload Song">
              ⬆ Upload Song
            </button>
          </div>
          <div className="song-list">
            {songLibrary.map((song) => (
              <div
                key={song.id}
                className={`song-item ${
                  selectedSong?.meta.title === song.title ? "selected" : ""
                }`}
              >
                <div 
                  className="song-item-content"
                  onClick={() => handleSelectSong(song.filename)}
                >
                  <div className="song-item-title">{song.title}</div>
                  <div className="song-item-artist">{song.artist}</div>
                </div>
                {!isDefaultSong(song.filename) && (
                  <button
                    className="delete-song-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteSong(song.filename, song.title);
                    }}
                    title="Delete song"
                  >
                    🗑
                  </button>
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="library-main">
          {loading && (
            <div className="loading-state">
              <div className="spinner"></div>
              <p>Loading song...</p>
            </div>
          )}

          {error && !loading && (
            <div className="error-state">
              <p>{error}</p>
            </div>
          )}

          {!loading && !error && !selectedSong && (
            <div className="empty-state">
              <p>Select a song from the library to view details</p>
            </div>
          )}

          {!loading && !error && selectedSong && (
            <div className="song-details">
              <div className="song-header">
                <div className="song-title-section">
                  <h1 className="song-title">{selectedSong.meta.title}</h1>
                  <h2 className="song-artist">{selectedSong.meta.artist}</h2>
                </div>
                <div className="song-links">
                  {selectedSong.meta.youtube && (
                    <button
                      className="link-button youtube"
                      onClick={() => openExternalLink(selectedSong.meta.youtube!)}
                      title="Open on YouTube"
                    >
                      <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
                      </svg>
                      YouTube
                    </button>
                  )}
                  {selectedSong.meta.spotify && (
                    <button
                      className="link-button spotify"
                      onClick={() => openExternalLink(selectedSong.meta.spotify!)}
                      title="Open on Spotify"
                    >
                      <svg
                        width="24"
                        height="24"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                      >
                        <path d="M12 0C5.4 0 0 5.4 0 12s5.4 12 12 12 12-5.4 12-12S18.66 0 12 0zm5.521 17.34c-.24.359-.66.48-1.021.24-2.82-1.74-6.36-2.101-10.561-1.141-.418.122-.779-.179-.899-.539-.12-.421.18-.78.54-.9 4.56-1.021 8.52-.6 11.64 1.32.42.18.479.659.301 1.02zm1.44-3.3c-.301.42-.841.6-1.262.3-3.239-1.98-8.159-2.58-11.939-1.38-.479.12-1.02-.12-1.14-.6-.12-.48.12-1.021.6-1.141C9.6 9.9 15 10.561 18.72 12.84c.361.181.54.78.241 1.2zm.12-3.36C15.24 8.4 8.82 8.16 5.16 9.301c-.6.179-1.2-.181-1.38-.721-.18-.601.18-1.2.72-1.381 4.26-1.26 11.28-1.02 15.721 1.621.539.3.719 1.02.419 1.56-.299.421-1.02.599-1.559.3z" />
                      </svg>
                      Spotify
                    </button>
                  )}
                </div>
              </div>

              <div className="song-metadata">
                <div className="metadata-item">
                  <span className="metadata-label">BPM</span>
                  <span className="metadata-value">{selectedSong.clock.bpm}</span>
                </div>
                <div className="metadata-item">
                  <span className="metadata-label">Time Signature</span>
                  {editingTimeSig ? (
                    <div className="metadata-value editable-time-sig">
                      <input
                        type="number"
                        className="time-sig-input"
                        value={timeSigNumerator}
                        onChange={(e) => setTimeSigNumerator(e.target.value)}
                        min="1"
                        max="32"
                        style={{ width: '40px' }}
                      />
                      <span>/</span>
                      <input
                        type="number"
                        className="time-sig-input"
                        value={timeSigDenominator}
                        onChange={(e) => setTimeSigDenominator(e.target.value)}
                        min="1"
                        max="32"
                        style={{ width: '40px' }}
                      />
                      <button onClick={handleSaveTimeSig} className="time-sig-btn save" title="Save">✓</button>
                      <button onClick={handleCancelEditTimeSig} className="time-sig-btn cancel" title="Cancel">✕</button>
                    </div>
                  ) : (
                    <span className="metadata-value editable" onClick={handleStartEditTimeSig} title="Click to edit">
                      {selectedSong.clock.timeSig[0]}/{selectedSong.clock.timeSig[1]} 
                      &nbsp; ({beatsPerBarQN(selectedSong.clock.timeSig)} quarter beats/bar)
                      <span className="edit-icon">✎</span>
                    </span>
                  )}
                </div>
                {selectedSong.clock.key && (
                  <div className="metadata-item">
                    <span className="metadata-label">Key</span>
                    <span className="metadata-value">{selectedSong.clock.key}</span>
                  </div>
                )}
                <div className="metadata-item">
                  <span className="metadata-label">Chords Used</span>
                  <span className="metadata-value">
                    {getUniqueChords(selectedSong).join(", ")}
                  </span>
                </div>
              </div>

              <div className="lyrics-section">
                <div className="lyrics-header">
                  <h3 className="section-title">Lyrics & Chords</h3>
                  <button
                    className="timeline-toggle"
                    onClick={() => setTimelineMode(timelineMode === 'beats' ? 'seconds' : 'beats')}
                    title={`Switch to ${timelineMode === 'beats' ? 'seconds' : 'beats'}`}
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                      <circle cx="12" cy="12" r="10"/>
                      <polyline points="12 6 12 12 16 14"/>
                    </svg>
                    {timelineMode === 'beats' ? 'Beats' : 'Time'}
                  </button>
                </div>
                {selectedSong.lyrics && selectedSong.lyrics.length > 0 ? (
                  <div className="lyrics-container">
                    {formatLyricsWithChords(selectedSong).map((line, index) => (
                      <div 
                        key={index} 
                        className="lyric-line-container"
                      >
                        <div className="lyric-line-wrapper">
                          <div className="timeline-ruler-inline">
                            {line.lineMarkers.map((marker, markerIndex) => (
                              <div
                                key={markerIndex}
                                className="timeline-tick-inline"
                                style={{ left: `${marker.position}%` }}
                              >
                                <span className="timeline-tick-label-inline">{marker.label}</span>
                                <span className="timeline-tick-line-inline"></span>
                              </div>
                            ))}
                          </div>
                          <div className="chord-line-wrapper">
                            {line.chords.map((chordInfo, chordIndex) => (
                              <span
                                key={chordIndex}
                                className="chord-marker"
                                style={{ left: `${chordInfo.position}%` }}
                              >
                                {chordInfo.chord}
                              </span>
                            ))}
                          </div>
                          <div className="lyric-line-wrapper">
                            {line.words.map((wordInfo, wordIndex) => (
                              <span
                                key={wordIndex}
                                className="lyric-word"
                                style={{ left: `${wordInfo.position}%` }}
                              >
                                {wordInfo.word}
                              </span>
                            ))}
                          </div>
                        </div>
                      </div>
                      ))}
                    </div>
                ) : (
                  <div className="no-lyrics">
                    <p>No lyrics available for this song</p>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      {showUploadDialog && uploadResult && (
        <SongUploadDialog 
          onClose={() => setShowUploadDialog(false)}
          songName={uploadResult.songName}
          isError={uploadResult.isError}
          errorMessage={uploadResult.errorMessage}
        />
      )}
    </div>
  );
}
