import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface SoundFontInfo {
  name: string;
  path: string;
  size_bytes: number;
}

interface SoundFontSelectorProps {
  onClose: () => void;
  onSelect: (name: string) => void;
}

export default function SoundFontSelector({ onClose, onSelect }: SoundFontSelectorProps) {
  const [soundfonts, setSoundfonts] = useState<SoundFontInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedName, setSelectedName] = useState<string | null>(null);
  const [uploading, setUploading] = useState(false);

  useEffect(() => {
    loadSoundFonts();
  }, []);

  const loadSoundFonts = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SoundFontInfo[]>("get_available_soundfonts");
      setSoundfonts(result);
    } catch (err: any) {
      setError(err.toString());
      console.error("Failed to load soundfonts:", err);
    } finally {
      setLoading(false);
    }
  };

  const handleUpload = async () => {
    try {
      setUploading(true);
      setError(null);
      
      // Open file dialog to select .sf2 file
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'SoundFont',
          extensions: ['sf2', 'SF2']
        }]
      });
      
      if (!selected || typeof selected !== 'string') {
        setUploading(false);
        return;
      }
      
      // Extract filename from path
      const fileName = selected.split(/[/\\]/).pop() || "unknown.sf2";
      
      // Call backend to upload/save the file
      const result = await invoke<string>("upload_soundfont", {
        filePath: selected,
        fileName: fileName
      });
      
      console.log(result);
      
      // Reload soundfonts to show the new one
      await loadSoundFonts();
      
    } catch (err: any) {
      setError(err.toString());
      console.error("Failed to upload soundfont:", err);
    } finally {
      setUploading(false);
    }
  };

  const handleSelect = async () => {
    if (!selectedName) return;
    
    try {
      await invoke("set_soundfont", { name: selectedName });
      onSelect(selectedName);
      onClose();
    } catch (err: any) {
      setError(err.toString());
      console.error("Failed to set soundfont:", err);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const getSoundFontDetails = (name: string): { icon: string; type: string; description: string } => {
    const lower = name.toLowerCase();
    
    if (lower.includes("bass")) {
      if (lower.includes("acoustic")) return { icon: "🎻", type: "Acoustic Bass", description: "Deep, warm acoustic bass tones" };
      if (lower.includes("rock")) return { icon: "🎸", type: "Rock Bass", description: "Punchy electric bass for rock" };
      if (lower.includes("synth")) return { icon: "🎹", type: "Bass Synth", description: "Classic synthesizer bass sounds" };
      return { icon: "🎸", type: "Bass Guitar", description: "Rich low-end bass frequencies" };
    }
    
    if (lower.includes("acoustic")) return { icon: "🎻", type: "Acoustic Guitar", description: "Natural, resonant acoustic sound" };
    if (lower.includes("funk")) return { icon: "⚡", type: "Funk Guitar", description: "Crisp, rhythmic funk tones" };
    if (lower.includes("jazz")) return { icon: "🎺", type: "Jazz Guitar", description: "Smooth, mellow jazz tones" };
    if (lower.includes("clean")) return { icon: "✨", type: "Clean Electric", description: "Clear, pristine electric sound" };
    if (lower.includes("rock") || lower.includes("60s")) return { icon: "🎸", type: "Rock Guitar", description: "Classic rock guitar tone" };
    if (lower.includes("palm") || lower.includes("muted")) return { icon: "🔇", type: "Palm Muted", description: "Tight, percussive muted sound" };
    if (lower.includes("12")) return { icon: "🎼", type: "12-String", description: "Rich, chorus-like 12-string tone" };
    if (lower.includes("electric") || lower.includes("ibanez")) return { icon: "⚡", type: "Electric Guitar", description: "Versatile electric guitar sound" };
    
    return { icon: "🎵", type: "Guitar", description: "High-quality guitar instrument" };
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Select SoundFont</h2>
          <button className="close-button" onClick={onClose}>✕</button>
        </div>

        <div className="modal-body">
          {loading && (
            <div className="loading-message">
              Loading SoundFonts...
            </div>
          )}

          {error && (
            <div className="error-message">
              Error: {error}
            </div>
          )}

          {!loading && !error && soundfonts.length === 0 && (
            <div className="empty-message">
              No SoundFonts found. Upload a .sf2 file or place files in the soundfont/ directory.
            </div>
          )}

          {!loading && !error && soundfonts.length > 0 && (
            <div className="soundfont-grid">
              {soundfonts.map((sf) => {
                const details = getSoundFontDetails(sf.name);
                return (
                  <div
                    key={sf.name}
                    className={`soundfont-tile ${selectedName === sf.name ? "selected" : ""}`}
                    onClick={() => setSelectedName(sf.name)}
                  >
                    {selectedName === sf.name && (
                      <div className="selected-indicator">✓</div>
                    )}
                    <div className="soundfont-icon">{details.icon}</div>
                    <div className="soundfont-type">{details.type}</div>
                    <div className="soundfont-name">{sf.name}</div>
                    <div className="soundfont-description">{details.description}</div>
                    <div className="soundfont-size">{formatBytes(sf.size_bytes)}</div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button 
            className="button-secondary" 
            onClick={handleUpload}
            disabled={uploading}
          >
            {uploading ? "Uploading..." : "📁 Upload SoundFont"}
          </button>
          <div style={{ flex: 1 }}></div>
          <button className="button-secondary" onClick={onClose}>
            Cancel
          </button>
          <button 
            className="button-primary" 
            onClick={handleSelect}
            disabled={!selectedName}
          >
            Apply
          </button>
        </div>
      </div>
    </div>
  );
}
