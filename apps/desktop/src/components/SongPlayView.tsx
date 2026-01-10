import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { open as openUrl } from "@tauri-apps/plugin-shell";
import SongUploadDialog from "./SongUploadDialog";
import "./SongPlayView.css";

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
    countInBars: number;
    subdivision?: string; // e.g., "8n" (eighth notes), "16n" (sixteenth notes)
  };
  mapping: {
    chords: Record<string, { frets: string[] }>;
  };
  lanes: Array<{
    name: string;
    events: ChordEvent[];
  }>;
  lyrics: LyricEvent[];
  sections: Section[];
}

interface ChordEvent {
  startBeat: number;
  dur: number;
  chord: string;
  section?: string;
}

interface LyricEvent {
  startBeat: number;
  text?: string; // Legacy format
  annotations?: Array<{ word: string; timeBeat: string }>; // New format
}

interface Section {
  name: string;
  fromBeat: number;
  toBeat: number;
}

interface TransportState {
  is_playing: boolean;
  current_beat: number;
  bpm: number;
  time_sig: [number, number];
  speed_multiplier: number;
  is_in_count_in: boolean;
}

interface ScoreData {
  score: number;
  combo: number;
  max_combo: number;
  hits: number;
  misses: number;
  accuracy: number;
  grade: string;
}

interface ControllerState {
  fret_green: boolean;
  fret_red: boolean;
  fret_yellow: boolean;
  fret_blue: boolean;
  fret_orange: boolean;
  strum_up: boolean;
  strum_down: boolean;
  whammy_bar: number;
  connected: boolean;
}

const FRET_COLORS = {
  GREEN: "#22c55e",
  RED: "#ef4444",
  YELLOW: "#eab308",
  BLUE: "#3b82f6",
  ORANGE: "#f97316",
};

const FRET_POSITIONS = {
  GREEN: 0,
  RED: 1,
  YELLOW: 2,
  BLUE: 3,
  ORANGE: 4,
};

const STRIKE_ZONE_SIZE = 60; // pixels above and below strike line for visual guidance

export default function SongPlayView() {
  const [chart, setChart] = useState<SongChart | null>(null);
  const [transport, setTransport] = useState<TransportState | null>(null);
  const [score, setScore] = useState<ScoreData | null>(null);
  const [controllerState, setControllerState] = useState<ControllerState | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [speedMultiplier, setSpeedMultiplier] = useState(1.0);
  const [songLibrary, setSongLibrary] = useState<SongLibraryEntry[]>([]);
  const [showLibrary, setShowLibrary] = useState(false);
  const [countdown, setCountdown] = useState<number | null>(null);
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [uploadResult, setUploadResult] = useState<{ songName: string; isError: boolean; errorMessage?: string } | null>(null);
  const [timelineMode, setTimelineMode] = useState<'beats' | 'seconds'>('beats');
  
  const prevStrumRef = useRef({ up: false, down: false });
  const animationRef = useRef<number | null>(null);

  useEffect(() => {
    // Reset song state when entering the view
    const initView = async () => {
      try {
        await invoke("song_stop");
        // Load chart first to get countInBars info
        await loadDefaultChart();
        await loadSongLibrary();
      } catch (error) {
        console.error("Failed to initialize view:", error);
      }
    };
    
    initView();
    
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

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

  const handleLoadFromLibrary = async (filename: string) => {
    try {
      setLoading(true);
      await invoke("song_stop");
      
      // Check if it's a default song
      const isDefaultSong = filename === "greensleeves.mitychart.json" || 
                           filename === "simple-blues.mitychart.json";
      
      if (isDefaultSong) {
        // Load from assets/songs
        if (filename === "greensleeves.mitychart.json") {
          await invoke("song_load_default_chart");
        } else {
          await invoke("song_load_chart_from_path", { path: `assets/songs/${filename}` });
        }
      } else {
        // Load from user library
        await invoke("song_load_from_library", { filename });
      }
      
      const chartJson = await invoke<string>("song_get_chart");
      if (chartJson) {
        const parsedChart = JSON.parse(chartJson);
        setChart(parsedChart);
        
        // Calculate count-in starting beat
        const countInBars = parsedChart.clock.countInBars || 0;
        const beatsPerBar = parsedChart.clock.timeSig[0];
        const countInBeats = countInBars * beatsPerBar;
        
        // Seek to beginning of count-in (negative beat)
        if (countInBeats > 0) {
          await invoke("song_seek", { beat: -countInBeats });
        }
      }
      setShowLibrary(false);
      setError(null);
    } catch (err) {
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
      await loadSongLibrary();
    } catch (err) {
      alert(`Failed to delete song: ${err}`);
    }
  };

  const loadSongLibrary = async () => {
    try {
      const library = await invoke<SongLibraryEntry[]>("song_list_library");
      
      // Always include default songs at the top
      const defaultSongs: SongLibraryEntry[] = [
        {
          id: "default-greensleeves",
          title: "Greensleeves",
          artist: "Traditional (Public Domain)",
          filename: "greensleeves.mitychart.json"
        },
        {
          id: "default-simple-blues",
          title: "Simple Blues",
          artist: "Traditional (Public Domain)",
          filename: "simple-blues.mitychart.json"
        }
      ];
      
      // Combine default songs with user library, filtering out duplicates
      const userLibrary = library.filter(
        song => !defaultSongs.some(def => def.filename === song.filename)
      );
      
      setSongLibrary([...defaultSongs, ...userLibrary]);
    } catch (err) {
      console.error("Failed to load song library:", err);
    }
  };

  useEffect(() => {
    // Start update loop
    const update = async () => {
      try {
        const [transportState, scoreData, ctrlState] = await Promise.all([
          invoke<TransportState>("song_get_transport_state"),
          invoke<ScoreData>("song_get_score"),
          invoke<ControllerState>("get_controller_state"),
        ]);
        
        setTransport(transportState);
        setScore(scoreData);
        setControllerState(ctrlState);

        // Check for strum events
        const prevStrum = prevStrumRef.current;
        if (ctrlState.strum_up && !prevStrum.up || ctrlState.strum_down && !prevStrum.down) {
          await handleStrum(ctrlState);
        }
        prevStrumRef.current = { up: ctrlState.strum_up, down: ctrlState.strum_down };

        // Update sustain
        const pressedFrets = getPressedFrets(ctrlState);
        await invoke("song_update_sustain", { pressedFrets });
      } catch (error) {
        console.error("Update error:", error);
      }

      animationRef.current = requestAnimationFrame(update);
    };

    animationRef.current = requestAnimationFrame(update);
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  const loadDefaultChart = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("song_stop");
      await invoke("song_load_default_chart");
      const chartJson = await invoke<string>("song_get_chart");
      if (chartJson) {
        const parsedChart = JSON.parse(chartJson);
        setChart(parsedChart);
        
        // Calculate count-in starting beat
        const countInBars = parsedChart.clock.countInBars || 0;
        const beatsPerBar = parsedChart.clock.timeSig[0];
        const countInBeats = countInBars * beatsPerBar;
        
        // Seek to beginning of count-in (negative beat)
        if (countInBeats > 0) {
          await invoke("song_seek", { beat: -countInBeats });
        }
      }
    } catch (error) {
      setError(`Failed to load chart: ${error}`);
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  const handlePlay = async () => {
    try {
      // Start countdown from 3
      let count = 3;
      setCountdown(count);
      
      // Countdown timer
      const countdownInterval = setInterval(() => {
        count--;
        if (count > 0) {
          setCountdown(count);
        } else {
          setCountdown(null);
          clearInterval(countdownInterval);
          // Actually start playing after countdown
          invoke("song_play").catch((error) => console.error("Play error:", error));
        }
      }, 1000);
    } catch (error) {
      console.error("Play error:", error);
    }
  };

  const handlePause = async () => {
    try {
      await invoke("song_pause");
    } catch (error) {
      console.error("Pause error:", error);
    }
  };

  const handleStop = async () => {
    try {
      await invoke("song_stop");
    } catch (error) {
      console.error("Stop error:", error);
    }
  };

  const handleSpeedChange = async (newSpeed: number) => {
    try {
      setSpeedMultiplier(newSpeed);
      await invoke("song_set_speed", { multiplier: newSpeed });
    } catch (error) {
      console.error("Speed change error:", error);
    }
  };

  const getPressedFrets = (state: ControllerState): string[] => {
    const frets: string[] = [];
    if (state.fret_green) frets.push("GREEN");
    if (state.fret_red) frets.push("RED");
    if (state.fret_yellow) frets.push("YELLOW");
    if (state.fret_blue) frets.push("BLUE");
    if (state.fret_orange) frets.push("ORANGE");
    return frets;
  };

  const handleStrum = async (state: ControllerState) => {
    const pressedFrets = getPressedFrets(state);
    try {
      const result = await invoke<{
        is_hit: boolean;
        chord?: string;
        accuracy?: number;
        miss_reason?: string;
      }>("song_check_strum", { pressedFrets });
      
      if (result.is_hit) {
        console.log(`Hit! ${result.chord} - Accuracy: ${(result.accuracy! * 100).toFixed(1)}%`);
      } else {
        console.log(`Miss: ${result.miss_reason}`);
      }
    } catch (error) {
      console.error("Strum check error:", error);
    }
  };

  const getCurrentLyric = (): string | null => {
    if (!chart || !transport) return null;
    
    const currentBeat = transport.current_beat;
    const lyric = chart.lyrics.find((l, i) => {
      const nextLyric = chart.lyrics[i + 1];
      const lyricBeat = l.startBeat ?? (l as any).beat ?? 0; // Support both formats
      const nextBeat = nextLyric ? (nextLyric.startBeat ?? (nextLyric as any).beat ?? Infinity) : Infinity;
      return lyricBeat <= currentBeat && currentBeat < nextBeat;
    });
    
    if (!lyric) return null;
    
    // Support both old and new lyrics formats
    if (lyric.annotations && lyric.annotations.length > 0) {
      // New format: find which words should be highlighted based on current beat
      const currentWords = lyric.annotations
        .filter(ann => parseFloat(ann.timeBeat) <= currentBeat)
        .map(ann => ann.word);
      return currentWords.length > 0 ? currentWords.join(' ') : null;
    }
    
    // Legacy format
    return lyric.text || null;
  };

  const getUpcomingEvents = (): ChordEvent[] => {
    if (!chart || !transport) return [];
    
    const currentBeat = transport.current_beat;
    const lookaheadBeats = 8; // Show 8 beats ahead
    
    return chart.lanes
      .flatMap(lane => lane.events)
      .filter(event => {
        const eventBeat = event.startBeat ?? (event as any).beat ?? 0;
        return eventBeat >= currentBeat && eventBeat < currentBeat + lookaheadBeats;
      })
      .sort((a, b) => {
        const beatA = a.startBeat ?? (a as any).beat ?? 0;
        const beatB = b.startBeat ?? (b as any).beat ?? 0;
        return beatA - beatB;
      });
  };

  const getCurrentSection = (): string => {
    if (!chart || !transport) return "";
    
    const currentBeat = transport.current_beat;
    const section = chart.sections.find(
      s => currentBeat >= s.fromBeat && currentBeat < s.toBeat
    );
    
    return section?.name || "";
  };

  const getTotalBeats = (): number => {
    if (!chart || chart.sections.length === 0) return 0;
    return Math.max(...chart.sections.map(s => s.toBeat));
  };

  const getProgressPercentage = (): number => {
    if (!transport) return 0;
    const total = getTotalBeats();
    if (total === 0) return 0;
    return Math.min(100, (transport.current_beat / total) * 100);
  };

  const beatsToSeconds = (beats: number): number => {
    if (!transport) return 0;
    // seconds = beats * (60 / bpm)
    return beats * (60 / transport.bpm);
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getCurrentDisplay = (): string => {
    if (!transport) return timelineMode === 'beats' ? 'Beat 0 / 0' : '0:00 / 0:00';
    
    // Handle count-in (negative beats)
    if (transport.current_beat < 0) {
      if (timelineMode === 'beats') {
        return `Count-in: ${Math.ceil(Math.abs(transport.current_beat))}`;
      } else {
        const countInSeconds = Math.abs(beatsToSeconds(transport.current_beat));
        return `Count-in: ${Math.ceil(countInSeconds)}s`;
      }
    }
    
    if (timelineMode === 'beats') {
      return `Beat ${Math.floor(transport.current_beat)} / ${getTotalBeats()}`;
    } else {
      const currentSeconds = beatsToSeconds(transport.current_beat);
      const totalSeconds = beatsToSeconds(getTotalBeats());
      return `${formatTime(currentSeconds)} / ${formatTime(totalSeconds)}`;
    }
  };

  if (loading) {
    return (
      <div className="song-play-view">
        <div className="loading">Loading song...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="song-play-view">
        <div className="error">{error}</div>
        <button onClick={loadDefaultChart}>Retry</button>
      </div>
    );
  }

  if (!chart || !transport || !score) {
    return (
      <div className="song-play-view">
        <div className="loading">Initializing...</div>
      </div>
    );
  }

  const currentLyric = getCurrentLyric();
  const upcomingEvents = getUpcomingEvents();
  const currentSection = getCurrentSection();

  return (
    <div className="song-play-view">
      <div className="controls-header">
        <div className="control-buttons-row">
          <button onClick={handleStop} className="control-btn">⏹ Stop</button>
          {transport.is_playing ? (
            <button onClick={handlePause} className="control-btn">⏸ Pause</button>
          ) : (
            <button onClick={handlePlay} className="control-btn">▶ Play</button>
          )}
          
          <div className="speed-controls">
            <label>Speed:</label>
            <button 
              onClick={() => handleSpeedChange(0.75)} 
              className={speedMultiplier === 0.75 ? "active" : ""}
            >
              0.75x
            </button>
            <button 
              onClick={() => handleSpeedChange(1.0)} 
              className={speedMultiplier === 1.0 ? "active" : ""}
            >
              1.0x
            </button>
            <button 
              onClick={() => handleSpeedChange(1.25)} 
              className={speedMultiplier === 1.25 ? "active" : ""}
            >
              1.25x
            </button>
          </div>

          <div className="beat-display">
            Beat: {transport.current_beat.toFixed(1)} / {transport.bpm} BPM
          </div>

          <button onClick={() => setShowLibrary(!showLibrary)} className="control-btn">
            📁 Song Library ({songLibrary.length})
          </button>
          <button onClick={handleUploadSong} className="control-btn">
            ⬆ Upload Song
          </button>
        </div>

        <div className="song-info-row">
          <div className="song-title-container">
            <h1>{chart.meta.title} - {chart.meta.artist}</h1>
            <div className="song-links">
              {chart.meta.youtube ? (
                <button 
                  onClick={() => openUrl(chart.meta.youtube!)}
                  className="song-link youtube-link"
                  title="Watch on YouTube"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M22.54 6.42a2.78 2.78 0 0 0-1.94-2C18.88 4 12 4 12 4s-6.88 0-8.6.46a2.78 2.78 0 0 0-1.94 2A29 29 0 0 0 1 11.75a29 29 0 0 0 .46 5.33A2.78 2.78 0 0 0 3.4 19c1.72.46 8.6.46 8.6.46s6.88 0 8.6-.46a2.78 2.78 0 0 0 1.94-2 29 29 0 0 0 .46-5.25 29 29 0 0 0-.46-5.33z"></path>
                    <polygon points="9.75 15.02 15.5 11.75 9.75 8.48 9.75 15.02"></polygon>
                  </svg>
                </button>
              ) : (
                <div className="song-link youtube-link disabled" title="YouTube link not available">
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M22.54 6.42a2.78 2.78 0 0 0-1.94-2C18.88 4 12 4 12 4s-6.88 0-8.6.46a2.78 2.78 0 0 0-1.94 2A29 29 0 0 0 1 11.75a29 29 0 0 0 .46 5.33A2.78 2.78 0 0 0 3.4 19c1.72.46 8.6.46 8.6.46s6.88 0 8.6-.46a2.78 2.78 0 0 0 1.94-2 29 29 0 0 0 .46-5.25 29 29 0 0 0-.46-5.33z"></path>
                    <polygon points="9.75 15.02 15.5 11.75 9.75 8.48 9.75 15.02"></polygon>
                  </svg>
                </div>
              )}
              {chart.meta.spotify ? (
                <button 
                  onClick={() => openUrl(chart.meta.spotify!)}
                  className="song-link spotify-link"
                  title="Listen on Spotify"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <path d="M8 14.5c2-1 4.5-1 6.5 0"></path>
                    <path d="M7.5 11.5c2.5-1.5 5.5-1.5 8 0"></path>
                    <path d="M7 8.5c3-1.5 6.5-1.5 9.5 0"></path>
                  </svg>
                </button>
              ) : (
                <div className="song-link spotify-link disabled" title="Spotify link not available">
                  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="12" cy="12" r="10"></circle>
                    <path d="M8 14.5c2-1 4.5-1 6.5 0"></path>
                    <path d="M7.5 11.5c2.5-1.5 5.5-1.5 8 0"></path>
                    <path d="M7 8.5c3-1.5 6.5-1.5 9.5 0"></path>
                  </svg>
                </div>
              )}
            </div>
          </div>
          <div className="section-indicator">{currentSection}</div>
        </div>
      </div>

      <div className="main-play-area">
        <div className="progress-bar-container">
          <div className="progress-info">
            <span>{getCurrentDisplay()}</span>
            <button 
              className="timeline-toggle-btn"
              onClick={() => setTimelineMode(timelineMode === 'beats' ? 'seconds' : 'beats')}
              title={`Switch to ${timelineMode === 'beats' ? 'time' : 'beats'} display`}
            >
              {timelineMode === 'beats' ? (
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="10"></circle>
                  <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
              ) : (
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M9 18V5l12-2v13"></path>
                  <circle cx="6" cy="18" r="3"></circle>
                  <circle cx="18" cy="16" r="3"></circle>
                </svg>
              )}
            </button>
            <span>{getProgressPercentage().toFixed(1)}%</span>
          </div>
          <div className="progress-bar">
            <div 
              className="progress-fill" 
              style={{ width: `${getProgressPercentage()}%` }}
            />
          </div>
        </div>
        
        <div className="highway-container">
          <ChordHighway
            events={upcomingEvents}
            currentBeat={transport.current_beat}
            chordMappings={chart.mapping.chords}
            controllerState={controllerState}
          />
          {countdown !== null && (
            <div className="countdown-overlay">
              <div key={countdown} className="countdown-number">{countdown}</div>
              <div key={`text-${countdown}`} className="countdown-text">Get Ready!</div>
            </div>
          )}
        </div>

        <div className="lyrics-display">
          {currentLyric && <div className="lyric-text">{currentLyric}</div>}
        </div>
      </div>

      <div className="score-display">
        <div className="score-item">
          <span className="score-label">Score:</span>
          <span className="score-value">{score.score.toLocaleString()}</span>
        </div>
        <div className="score-item">
          <span className="score-label">Combo:</span>
          <span className="score-value combo">{score.combo}x</span>
        </div>
        <div className="score-item">
          <span className="score-label">Accuracy:</span>
          <span className="score-value">{score.accuracy.toFixed(1)}%</span>
        </div>
        <div className="score-item">
          <span className="score-label">Grade:</span>
          <span className="score-value grade">{score.grade}</span>
        </div>
      </div>

      {showLibrary && (
        <div className="song-library-modal">
          <div className="library-header">
            <h3>Song Library</h3>
            <button onClick={() => setShowLibrary(false)} className="close-btn">✕</button>
          </div>
          <div className="library-list">
            {songLibrary.length === 0 ? (
              <div className="empty-library">No songs in library. Upload a song to get started!</div>
            ) : (
              songLibrary.map((song) => {
                const isDefaultSong = song.filename === "greensleeves.mitychart.json" || 
                                     song.filename === "simple-blues.mitychart.json";
                return (
                  <div key={song.id} className="library-item">
                    <div className="song-info">
                      <div className="song-title">
                        {song.title}
                        {isDefaultSong && <span className="default-badge">Default</span>}
                      </div>
                      <div className="song-artist">{song.artist}</div>
                    </div>
                    <div className="song-actions">
                      <button 
                        onClick={() => handleLoadFromLibrary(song.filename)}
                        className="action-btn load-btn"
                      >
                        ▶ Load
                      </button>
                      {!isDefaultSong && (
                        <button 
                          onClick={() => handleDeleteSong(song.filename, song.title)}
                          className="action-btn delete-btn"
                        >
                          🗑
                        </button>
                      )}
                    </div>
                  </div>
                );
              })
            )}
          </div>
        </div>
      )}

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

interface ChordHighwayProps {
  events: ChordEvent[];
  currentBeat: number;
  chordMappings: Record<string, { frets: string[] }>;
  controllerState: ControllerState | null;
}

function ChordHighway({ events, currentBeat, chordMappings, controllerState }: ChordHighwayProps) {
  const BEAT_HEIGHT = 80; // pixels per beat
  const HIGHWAY_HEIGHT = 600;
  const STRIKE_LINE_POSITION = HIGHWAY_HEIGHT - 120;
  const STRIKE_ZONE_TOP = STRIKE_LINE_POSITION - STRIKE_ZONE_SIZE;

  return (
    <div className="chord-highway" style={{ height: HIGHWAY_HEIGHT }}>
      {/* Fret lanes */}
      {["GREEN", "RED", "YELLOW", "BLUE", "ORANGE"].map((fret) => (
        <div
          key={fret}
          className={`fret-lane ${controllerState?.[`fret_${fret.toLowerCase()}` as keyof ControllerState] ? "active" : ""}`}
          style={{
            left: `${FRET_POSITIONS[fret as keyof typeof FRET_POSITIONS] * 20}%`,
            width: "20%",
            backgroundColor: `${FRET_COLORS[fret as keyof typeof FRET_COLORS]}20`,
          }}
        />
      ))}

      {/* Strike zone indicator - visual guide for timing */}
      <div className="strike-zone" style={{ 
        top: STRIKE_ZONE_TOP, 
        height: STRIKE_ZONE_SIZE * 2,
      }} />

      {/* Strike line - STRUM HERE! */}
      <div className="strike-line" style={{ top: STRIKE_LINE_POSITION }}>
        <div className="strike-line-label">▼ STRUM HERE ▼</div>
      </div>

      {/* Chord notes */}
      {events.map((event, i) => {
        const mapping = chordMappings[event.chord];
        if (!mapping) return null;

        const eventBeat = event.startBeat ?? (event as any).beat ?? 0;
        const beatOffset = eventBeat - currentBeat;
        const yPosition = STRIKE_LINE_POSITION - beatOffset * BEAT_HEIGHT;

        // Don't render if too far off screen
        if (yPosition < -100 || yPosition > HIGHWAY_HEIGHT + 100) return null;

        return (
          <div key={i}>
            {mapping.frets.map((fret, j) => {
              const xPosition = FRET_POSITIONS[fret as keyof typeof FRET_POSITIONS] * 20;
              const isSustain = event.dur >= 2.0;
              const sustainHeight = isSustain ? event.dur * BEAT_HEIGHT : 0;

              return (
                <div key={j}>
                  {/* Sustain tail */}
                  {isSustain && (
                    <div
                      className="sustain-tail"
                      style={{
                        left: `${xPosition + 2.5}%`,
                        top: yPosition - sustainHeight,
                        width: "15%",
                        height: sustainHeight,
                        backgroundColor: FRET_COLORS[fret as keyof typeof FRET_COLORS],
                        opacity: 0.4,
                      }}
                    />
                  )}
                  {/* Note head */}
                  <div
                    className="note-head"
                    style={{
                      left: `${xPosition}%`,
                      top: yPosition,
                      width: "20%",
                      backgroundColor: FRET_COLORS[fret as keyof typeof FRET_COLORS],
                      border: `2px solid ${FRET_COLORS[fret as keyof typeof FRET_COLORS]}`,
                    }}
                  >
                    <div className="chord-label">{event.chord}</div>
                  </div>
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}
