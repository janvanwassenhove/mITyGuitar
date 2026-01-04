import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import MenuBar from "./components/MenuBar";
import LiveView from "./components/LiveView";
import DiagnosticsView from "./components/DiagnosticsView";
import ProfileManagerView from "./components/ProfileManagerView";
import SongPlayView from "./components/SongPlayView";
import SongLibraryView from "./components/SongLibraryView";
import SoundFontSelector from "./components/SoundFontSelector";
import VirtualInstrumentSelector from "./components/VirtualInstrumentSelector";
import AboutDialog from "./components/AboutDialog";
import StartupMenu from "./components/StartupMenu";
import AudioSettings from "./components/AudioSettings";
// Import CSS for new components
import "./components/FretButton.css";
import "./components/FretBoard.css";
import "./components/ChordMappingControls.css";

type View = "live" | "diagnostics" | "mapping-wizard" | "profile-manager" | "song-play" | "song-library" | "audio-settings";

function App() {
  const [showStartupMenu, setShowStartupMenu] = useState<boolean>(true);
  const [currentView, setCurrentView] = useState<View>("live");
  const [genreInfo, setGenreInfo] = useState<any>(null);
  const [showSoundFontSelector, setShowSoundFontSelector] = useState(false);
  const [showVirtualInstrumentSelector, setShowVirtualInstrumentSelector] = useState(false);
  const [showAboutDialog, setShowAboutDialog] = useState(false);
  const [isRescanningSoundFonts, setIsRescanningSoundFonts] = useState(false);

  useEffect(() => {
    // Only load genre info and start audio health check if startup menu is not shown
    if (!showStartupMenu) {
      loadGenreInfo();
      
      // Periodically check audio health and reconnect if needed
      const audioHealthCheck = setInterval(async () => {
        try {
          const reconnected = await invoke<boolean>("check_audio_health");
          if (reconnected) {
            console.log("Audio device reconnected automatically");
          }
        } catch (error) {
          console.error("Audio health check failed:", error);
        }
      }, 5000); // Check every 5 seconds
      
      return () => clearInterval(audioHealthCheck);
    }
  }, [showStartupMenu]);

  const loadGenreInfo = async () => {
    try {
      const info = await invoke("get_current_genre_info");
      setGenreInfo(info);
    } catch (error) {
      console.error("Failed to load genre info:", error);
    }
  };

  const handleMenuAction = async (action: string) => {
    try {
      switch (action) {
        case "view_live":
          setCurrentView("live");
          break;
        case "view_diagnostics":
          setCurrentView("diagnostics");
          break;
        case "view_profile_manager":
          setCurrentView("profile-manager");
          break;
        case "view_song_play":
          setCurrentView("song-play");
          break;
        case "view_song_library":
          setCurrentView("song-library");
          break;
        case "choose_soundfont":
          setShowSoundFontSelector(true);
          break;
        case "choose_virtual_instrument":
          setShowVirtualInstrumentSelector(true);
          break;
        case "audio_settings":
          setCurrentView("audio-settings");
          break;
        case "rescan_soundfonts":
          setIsRescanningSoundFonts(true);
          try {
            await invoke("rescan_soundfonts");
            console.log("SoundFonts rescanned");
            // Keep the loading state visible for a moment so user sees feedback
            setTimeout(() => setIsRescanningSoundFonts(false), 800);
          } catch (error) {
            console.error("Failed to rescan soundfonts:", error);
            setIsRescanningSoundFonts(false);
          }
          break;
        case "next_instrument":
          await invoke("next_instrument");
          break;
        case "prev_instrument":
          await invoke("prev_instrument");
          break;
        case "next_pattern":
          await invoke("next_pattern");
          await loadGenreInfo();
          break;
        case "prev_pattern":
          await invoke("prev_pattern");
          await loadGenreInfo();
          break;
        case "panic":
          await invoke("panic_all_notes_off");
          break;
        case "quit":
          await invoke("quit_app");
          break;
        case "genre_punk":
          await invoke("set_genre", { genreName: "punk" });
          await loadGenreInfo();
          break;
        case "genre_rock":
          await invoke("set_genre", { genreName: "rock" });
          await loadGenreInfo();
          break;
        case "genre_edm":
          await invoke("set_genre", { genreName: "edm" });
          await loadGenreInfo();
          break;
        case "about":
          setShowAboutDialog(true);
          break;
      }
    } catch (error) {
      console.error("Menu action failed:", error);
    }
  };

  const handleStartupSelection = (view: "live" | "song-play") => {
    setCurrentView(view);
    setShowStartupMenu(false);
  };

  // Show startup menu if not yet dismissed
  if (showStartupMenu) {
    return <StartupMenu onSelectView={handleStartupSelection} />;
  }

  return (
    <div className="app-container">
      <MenuBar 
        onMenuAction={handleMenuAction} 
        currentView={currentView}
        isRescanningSoundFonts={isRescanningSoundFonts}
      />
      
      <main className="main-content">
        {currentView === "live" && (
          <LiveView 
            genreInfo={genreInfo} 
            onAction={handleMenuAction}
          />
        )}
        {currentView === "diagnostics" && <DiagnosticsView />}
        {currentView === "profile-manager" && <ProfileManagerView />}
        {currentView === "song-play" && <SongPlayView />}
        {currentView === "song-library" && <SongLibraryView />}
        {currentView === "audio-settings" && <AudioSettings />}
      </main>

      {isRescanningSoundFonts && (
        <div className="loading-overlay">
          <div className="loading-content">
            <div className="loading-spinner"></div>
            <h2>Rescanning SoundFonts...</h2>
            <p>Loading instruments</p>
          </div>
        </div>
      )}

      {showSoundFontSelector && (
        <SoundFontSelector
          onClose={() => setShowSoundFontSelector(false)}
          onSelect={(name) => {
            console.log("Selected SoundFont:", name);
            setShowSoundFontSelector(false);
          }}
        />
      )}
      {showVirtualInstrumentSelector && (
        <VirtualInstrumentSelector
          onClose={() => setShowVirtualInstrumentSelector(false)}
          onSelect={(id) => {
            console.log("Selected virtual instrument:", id);
            setShowVirtualInstrumentSelector(false);
          }}
        />
      )}

      {showAboutDialog && (
        <AboutDialog onClose={() => setShowAboutDialog(false)} />
      )}
    </div>
  );
}

export default App;
