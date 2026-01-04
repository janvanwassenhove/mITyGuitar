import React from 'react';
import FretButton from './FretButton';
import './FretBoard.css';

interface ChordMap {
  green: string;
  red: string;
  yellow: string;
  blue: string;
  orange: string;
}

interface ControllerState {
  fret_green: boolean;
  fret_red: boolean;
  fret_blue: boolean;
  fret_yellow: boolean;
  fret_orange: boolean;
  solo_green: boolean;
  solo_red: boolean;
  solo_blue: boolean;
  solo_yellow: boolean;
  solo_orange: boolean;
}

interface FretBoardProps {
  mainChords: ChordMap;
  soloChords: ChordMap;
  controllerState?: ControllerState;
  isEditable?: boolean;
  onChordEdit?: (fret: keyof ChordMap, row: 'main' | 'solo', newChord: string) => void;
}

export default function FretBoard({ 
  mainChords, 
  soloChords, 
  controllerState,
  isEditable = true,
  onChordEdit 
}: FretBoardProps) {
  const fretButtons: Array<keyof ChordMap> = ['green', 'red', 'yellow', 'blue', 'orange'];

  const handleChordEdit = (fret: keyof ChordMap, row: 'main' | 'solo') => {
    return (newChord: string) => {
      if (onChordEdit) {
        onChordEdit(fret, row, newChord);
      }
    };
  };

  return (
    <div className="fret-board">
      <div className="fret-board-header">
        <h3>Chord Mapping</h3>
      </div>
      
      <div className="fret-rows">
        {/* Solo Row */}
        <div className="fret-row-container">
          <span className="row-label solo-label">SOLO</span>
          <div className="fret-row solo-row">
            {fretButtons.map(fret => (
              <FretButton
                key={`solo-${fret}`}
                fretButton={fret}
                chordLabel={soloChords[fret]}
                isPressed={controllerState?.[`solo_${fret}` as keyof ControllerState] || false}
                row="solo"
                isEditable={isEditable}
                onChordEdit={handleChordEdit(fret, 'solo')}
              />
            ))}
          </div>
        </div>
        
        {/* Main Row */}
        <div className="fret-row-container">
          <span className="row-label main-label">MAIN</span>
          <div className="fret-row main-row">
            {fretButtons.map(fret => (
              <FretButton
                key={`main-${fret}`}
                fretButton={fret}
                chordLabel={mainChords[fret]}
                isPressed={controllerState?.[`fret_${fret}` as keyof ControllerState] || false}
                row="main"
                isEditable={isEditable}
                onChordEdit={handleChordEdit(fret, 'main')}
              />
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}