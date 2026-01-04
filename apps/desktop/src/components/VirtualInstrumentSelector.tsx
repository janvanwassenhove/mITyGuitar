import { useState } from "react";

interface VirtualInstrument {
  id: string;
  name: string;
  type: string;
  icon: string;
  description: string;
}

interface VirtualInstrumentSelectorProps {
  onClose: () => void;
  onSelect: (id: string) => void;
}

const virtualInstruments: VirtualInstrument[] = [
  {
    id: "acoustic_guitar",
    name: "Acoustic Guitar",
    type: "String",
    icon: "🎻",
    description: "Warm, natural acoustic guitar with rich harmonics"
  },
  {
    id: "electric_guitar",
    name: "Electric Guitar",
    type: "String",
    icon: "⚡",
    description: "Versatile electric guitar with classic tone"
  },
  {
    id: "bass_guitar",
    name: "Bass Guitar",
    type: "Bass",
    icon: "🎸",
    description: "Deep, powerful bass frequencies"
  },
  {
    id: "jazz_guitar",
    name: "Jazz Guitar",
    type: "String",
    icon: "🎺",
    description: "Smooth, mellow jazz guitar tones"
  },
  {
    id: "funk_guitar",
    name: "Funk Guitar",
    type: "String",
    icon: "⚡",
    description: "Crisp, rhythmic funk guitar"
  },
  {
    id: "rock_guitar",
    name: "Rock Guitar",
    type: "String",
    icon: "🎸",
    description: "Powerful rock guitar with sustain"
  },
  {
    id: "classical_guitar",
    name: "Classical Guitar",
    type: "String",
    icon: "🎼",
    description: "Soft, elegant classical nylon strings"
  },
  {
    id: "12string_guitar",
    name: "12-String Guitar",
    type: "String",
    icon: "🎼",
    description: "Rich, chorus-like 12-string harmonics"
  },
  {
    id: "distortion_guitar",
    name: "Distortion Guitar",
    type: "String",
    icon: "🔥",
    description: "Heavy, distorted guitar for metal"
  },
  {
    id: "clean_guitar",
    name: "Clean Guitar",
    type: "String",
    icon: "✨",
    description: "Crystal clear, pristine electric tone"
  },
  {
    id: "palm_muted",
    name: "Palm Muted",
    type: "String",
    icon: "🔇",
    description: "Tight, percussive palm muted sound"
  },
  {
    id: "synth_bass",
    name: "Synth Bass",
    type: "Synth",
    icon: "🎹",
    description: "Classic synthesizer bass tones"
  }
];

export default function VirtualInstrumentSelector({ onClose, onSelect }: VirtualInstrumentSelectorProps) {
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const handleSelect = () => {
    if (!selectedId) return;
    onSelect(selectedId);
    onClose();
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Virtual Instruments</h2>
          <button className="close-button" onClick={onClose}>✕</button>
        </div>

        <div className="modal-body">
          <div className="soundfont-grid">
            {virtualInstruments.map((instrument) => (
              <div
                key={instrument.id}
                className={`soundfont-tile ${selectedId === instrument.id ? "selected" : ""}`}
                onClick={() => setSelectedId(instrument.id)}
              >
                {selectedId === instrument.id && (
                  <div className="selected-indicator">✓</div>
                )}
                <div className="soundfont-icon">{instrument.icon}</div>
                <div className="soundfont-type">{instrument.type}</div>
                <div className="soundfont-name">{instrument.name}</div>
                <div className="soundfont-description">{instrument.description}</div>
              </div>
            ))}
          </div>
        </div>

        <div className="modal-footer">
          <button className="button-secondary" onClick={onClose}>
            Cancel
          </button>
          <button 
            className="button-primary" 
            onClick={handleSelect}
            disabled={!selectedId}
          >
            Apply
          </button>
        </div>
      </div>
    </div>
  );
}
