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
  annotations?: Array<{ word: string; timeBeat: string }>; // New format
}

interface LyricLine {
  lyrics: string;
  words: Array<{ word: string; position: number }>; // Individual word positions
  startBeat: number;
  endBeat: number;
  timelinePosition: number;
  width: number;
  chords: Array<{ position: number; chord: string }>;
  lineMarkers: Array<{ beat: number; label: string; position: number }>;
}

export default function SongLibraryView() {
  const [songLibrary, setSongLibrary] = useState<SongLibraryEntry[]>([]);
  const [selectedSong, setSelectedSong] = useState<SongChart | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [timelineMode, setTimelineMode] = useState<'beats' | 'seconds'>('beats');
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [uploadResult, setUploadResult] = useState<{ songName: string; isError: boolean; errorMessage?: string } | null>(null);

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

      const path = typeof selected === "string" ? selected : selected.path;
      
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

  const beatsToSeconds = (beats: number, bpm: number, countInBars: number = 0, timeSig: [number, number] = [4, 4]): number => {
    // Count-in offset: countInBeats = countInBars * timeSigTop
    const countInBeats = countInBars * timeSig[0];
    const totalBeats = beats + countInBeats;
    // secondsPerBeat = 60 / bpm
    // timeSec = beat * secondsPerBeat
    return (totalBeats / bpm) * 60;
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getTimeLabel = (beat: number, bpm: number, countInBars: number = 0, timeSig: [number, number] = [4, 4]): string => {
    if (timelineMode === 'beats') {
      return `${beat}`;
    } else {
      const seconds = beatsToSeconds(beat, bpm, countInBars, timeSig);
      return formatTime(seconds);
    }
  };

  const getLineTimelineMarkers = (startBeat: number, endBeat: number, bpm: number, countInBars: number = 0, timeSig: [number, number] = [4, 4]) => {
    const markers: Array<{ beat: number; label: string; position: number }> = [];
    const lineRange = endBeat - startBeat;
    const interval = lineRange > 16 ? 8 : (lineRange > 8 ? 4 : 2); // Adaptive interval
    
    for (let beat = Math.ceil(startBeat / interval) * interval; beat <= endBeat; beat += interval) {
      if (beat >= startBeat && beat <= endBeat) {
        const position = ((beat - startBeat) / lineRange) * 100;
        markers.push({ 
          beat, 
          label: getTimeLabel(beat, bpm, countInBars, timeSig),
          position: Math.max(0, Math.min(100, position))
        });
      }
    }
    
    return markers;
  };

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

    const minBeat = Math.min(...sortedLyrics.map(l => l.startBeat ?? (l as any).beat ?? 0));
    const maxBeat = Math.max(...sortedLyrics.map(l => l.startBeat ?? (l as any).beat ?? 0));
    const totalRange = maxBeat - minBeat;

    const lines: LyricLine[] = [];

    sortedLyrics.forEach((lyric, lyricIndex) => {
      const lyricBeat = lyric.startBeat ?? (lyric as any).beat ?? 0;
      
      // Determine the end beat for this line
      let lineEndBeat: number;
      if (lyric.annotations && lyric.annotations.length > 0) {
        // Use the last annotation's timeBeat as the end
        const lastAnnotation = lyric.annotations[lyric.annotations.length - 1];
        lineEndBeat = parseFloat(lastAnnotation.timeBeat) + 0.5; // Add a small buffer
      } else {
        // Fallback to next lyric's start or max beat
        lineEndBeat = lyricIndex < sortedLyrics.length - 1 
          ? (sortedLyrics[lyricIndex + 1].startBeat ?? (sortedLyrics[lyricIndex + 1] as any).beat ?? maxBeat + 8)
          : maxBeat + 8;
      }

      // Process words with individual positions
      const words: Array<{ word: string; position: number }> = [];
      let lyricText = '';
      
      if (lyric.annotations && lyric.annotations.length > 0) {
        const lineRange = lineEndBeat - lyricBeat;
        lyric.annotations.forEach(ann => {
          const wordBeat = parseFloat(ann.timeBeat);
          const wordOffset = wordBeat - lyricBeat;
          const position = lineRange > 0 ? (wordOffset / lineRange) * 100 : 0;
          words.push({
            word: ann.word,
            position: Math.max(0, Math.min(95, position))
          });
        });
        lyricText = lyric.annotations.map(ann => ann.word).join(' ');
      } else {
        // Legacy format without annotations
        lyricText = lyric.text || '';
        words.push({ word: lyricText, position: 0 });
      }

      const chordPositions: Array<{ position: number; chord: string }> = [];
      const nextLyricBeat = lineEndBeat;

      sortedChords.forEach((chordEvent) => {
        const chordBeat = chordEvent.startBeat ?? (chordEvent as any).beat ?? 0;
        if (chordBeat >= lyricBeat && chordBeat < nextLyricBeat) {
          // Calculate position relative to the current line span
          const lineRange = nextLyricBeat - lyricBeat;
          const chordOffset = chordBeat - lyricBeat;
          const position = lineRange > 0 ? (chordOffset / lineRange) * 100 : 0;
          
          chordPositions.push({
            position: Math.max(0, Math.min(95, position)),
            chord: chordEvent.chord
          });
        }
      });

      // Calculate this line's position on the global timeline
      const timelinePosition = ((lyricBeat - minBeat) / totalRange) * 100;
      const lineWidth = ((nextLyricBeat - lyricBeat) / totalRange) * 100;

      // Generate timeline markers for this line
      const lineMarkers = getLineTimelineMarkers(
        lyricBeat, 
        nextLyricBeat, 
        chart.clock.bpm,
        chart.clock.countInBars ?? 0,
        chart.clock.timeSig ?? [4, 4]
      );

      console.log(`Line ${lyricIndex}: beat ${lyricBeat}-${nextLyricBeat}, markers: ${lineMarkers.length}, chords: ${chordPositions.length}`);

      lines.push({
        lyrics: lyricText,
        words,
        startBeat: lyricBeat,
        endBeat: nextLyricBeat,
        timelinePosition: Math.max(0, Math.min(100, timelinePosition)),
        width: Math.max(0, Math.min(100, lineWidth)),
        chords: chordPositions.sort((a, b) => a.position - b.position),
        lineMarkers
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
                  <span className="metadata-value">
                    {selectedSong.clock.timeSig[0]}/{selectedSong.clock.timeSig[1]}
                  </span>
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
                          <div className="lyric-text">{line.lyrics}</div>
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
