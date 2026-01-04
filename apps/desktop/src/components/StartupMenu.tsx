import { useState } from "react";
import "./StartupMenu.css";
import logo from "../assets/logo.png";

interface StartupMenuProps {
  onSelectView: (view: "live" | "song-play") => void;
}

function StartupMenu({ onSelectView }: StartupMenuProps) {
  const [selectedOption, setSelectedOption] = useState<"live" | "song-play" | null>(null);

  const handleSelect = (view: "live" | "song-play") => {
    setSelectedOption(view);
    // Small delay for visual feedback
    setTimeout(() => {
      onSelectView(view);
    }, 200);
  };

  return (
    <div className="startup-menu">
      <div className="startup-menu-container">        <div className="startup-logo">
          <img src={logo} alt="MityGuitar Logo" />
        </div>       
        <p className="startup-subtitle">Choose Your Mode</p>
        
        <div className="startup-options">
          <button
            className={`startup-option ${selectedOption === "live" ? "selected" : ""}`}
            onClick={() => handleSelect("live")}
          >
            <div className="option-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M9 18V5l12-2v13"></path>
                <circle cx="6" cy="18" r="3"></circle>
                <circle cx="18" cy="16" r="3"></circle>
              </svg>
            </div>
            <h2>Live Mode</h2>
            <p>Play freely with real-time controller input and chord mapping</p>
          </button>

          <button
            className={`startup-option ${selectedOption === "song-play" ? "selected" : ""}`}
            onClick={() => handleSelect("song-play")}
          >
            <div className="option-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polygon points="5 3 19 12 5 21 5 3"></polygon>
              </svg>
            </div>
            <h2>Song Mode</h2>
            <p>Load and play along with songs, tracks, and charts</p>
          </button>
        </div>

        <div className="startup-footer">
          <p>You can switch modes anytime from the View menu</p>
        </div>
      </div>
    </div>
  );
}

export default StartupMenu;
