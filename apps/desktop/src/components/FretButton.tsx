import React, { useState } from 'react';
import './FretButton.css';

interface FretButtonProps {
  fretButton: 'green' | 'red' | 'yellow' | 'blue' | 'orange';
  chordLabel: string;
  isPressed: boolean;
  row: 'main' | 'solo';
  isEditable?: boolean;
  onChordEdit?: (newChord: string) => void;
}

const FRET_COLORS = {
  green: '#4ade80',
  red: '#ef4444', 
  yellow: '#eab308',
  blue: '#3b82f6',
  orange: '#f97316'
};

// Valid chord options grouped by root note
const VALID_CHORDS = [
  // Power chords
  'C5', 'C#5', 'D5', 'D#5', 'E5', 'F5', 'F#5', 'G5', 'G#5', 'A5', 'A#5', 'B5',
  // Major chords
  'C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B',
  // Minor chords
  'Cm', 'C#m', 'Dm', 'D#m', 'Em', 'Fm', 'F#m', 'Gm', 'G#m', 'Am', 'A#m', 'Bm',
  // Suspended chords
  'Csus2', 'C#sus2', 'Dsus2', 'D#sus2', 'Esus2', 'Fsus2', 'F#sus2', 'Gsus2', 'G#sus2', 'Asus2', 'A#sus2', 'Bsus2',
  'Csus4', 'C#sus4', 'Dsus4', 'D#sus4', 'Esus4', 'Fsus4', 'F#sus4', 'Gsus4', 'G#sus4', 'Asus4', 'A#sus4', 'Bsus4',
  // Add9 chords
  'Cadd9', 'C#add9', 'Dadd9', 'D#add9', 'Eadd9', 'Fadd9', 'F#add9', 'Gadd9', 'G#add9', 'Aadd9', 'A#add9', 'Badd9',
  // 7th chords
  'C7', 'C#7', 'D7', 'D#7', 'E7', 'F7', 'F#7', 'G7', 'G#7', 'A7', 'A#7', 'B7',
  'Cmaj7', 'C#maj7', 'Dmaj7', 'D#maj7', 'Emaj7', 'Fmaj7', 'F#maj7', 'Gmaj7', 'G#maj7', 'Amaj7', 'A#maj7', 'Bmaj7',
  'Cm7', 'C#m7', 'Dm7', 'D#m7', 'Em7', 'Fm7', 'F#m7', 'Gm7', 'G#m7', 'Am7', 'A#m7', 'Bm7',
];

export default function FretButton({ 
  fretButton, 
  chordLabel, 
  isPressed, 
  row,
  isEditable = true,
  onChordEdit 
}: FretButtonProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(chordLabel);

  const handleChordClick = () => {
    if (isEditable && onChordEdit) {
      setIsEditing(true);
      setEditValue(chordLabel);
    }
  };

  const handleEditSubmit = (newValue: string) => {
    if (onChordEdit && newValue) {
      onChordEdit(newValue);
    }
    setIsEditing(false);
  };

  const handleEditCancel = () => {
    setEditValue(chordLabel);
    setIsEditing(false);
  };

  const handleSelectChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const newValue = e.target.value;
    setEditValue(newValue);
    handleEditSubmit(newValue);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      handleEditCancel();
    }
  };

  return (
    <div 
      className={`fret-button ${row} ${isPressed ? 'pressed' : ''}`}
      style={{ '--fret-color': FRET_COLORS[fretButton] } as React.CSSProperties}
    >
      <div className="fret-circle" onClick={handleChordClick}>
        {isEditing ? (
          <select
            value={editValue}
            onChange={handleSelectChange}
            onBlur={() => setIsEditing(false)}
            onKeyDown={handleKeyDown}
            className="chord-edit-select"
            autoFocus
          >
            <option value="">Select chord...</option>
            {VALID_CHORDS.map(chord => (
              <option key={chord} value={chord}>{chord}</option>
            ))}
          </select>
        ) : (
          <div 
            className={`chord-label ${isEditable ? 'editable' : ''}`}
            title={isEditable ? 'Click to edit chord' : ''}
          >
            {chordLabel || '—'}
          </div>
        )}
      </div>
    </div>
  );
}