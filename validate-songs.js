// Song Validation Tool
// This script will help identify chord mapping issues in song files

const fs = require('fs');
const path = require('path');

function validateSongFile(filePath) {
    try {
        const content = fs.readFileSync(filePath, 'utf8');
        const song = JSON.parse(content);
        
        console.log(`\n=== Validating: ${path.basename(filePath)} ===`);
        
        // Get all defined chords from mapping
        const mappedChords = new Set();
        if (song.mapping && song.mapping.chords) {
            Object.keys(song.mapping.chords).forEach(chord => {
                mappedChords.add(chord);
                console.log(`✓ Mapped chord: "${chord}"`);
            });
        }
        
        // Get all used chords from lanes
        const usedChords = new Set();
        if (song.lanes) {
            song.lanes.forEach(lane => {
                if (lane.events) {
                    lane.events.forEach(event => {
                        if (event.chord) {
                            usedChords.add(event.chord);
                        }
                    });
                }
            });
        }
        
        console.log(`\nUsed chords:`);
        usedChords.forEach(chord => console.log(`  - "${chord}"`));
        
        // Find missing mappings
        const missingChords = [];
        usedChords.forEach(chord => {
            if (!mappedChords.has(chord)) {
                missingChords.push(chord);
            }
        });
        
        if (missingChords.length > 0) {
            console.log(`\n❌ MISSING CHORD MAPPINGS:`);
            missingChords.forEach(chord => {
                console.log(`  - "${chord}" (used in events but not mapped)`);
            });
            return false;
        } else {
            console.log(`\n✅ All chords are properly mapped!`);
            return true;
        }
        
    } catch (error) {
        console.error(`Error validating ${filePath}:`, error.message);
        return false;
    }
}

// Validate all song files
const songsDir = './assets/songs';
if (fs.existsSync(songsDir)) {
    const files = fs.readdirSync(songsDir)
        .filter(file => file.endsWith('.json'))
        .map(file => path.join(songsDir, file));
    
    console.log('🎵 Song Chord Validation Tool');
    console.log('============================');
    
    let hasErrors = false;
    files.forEach(file => {
        const isValid = validateSongFile(file);
        if (!isValid) hasErrors = true;
    });
    
    if (hasErrors) {
        console.log('\n❌ Found chord mapping issues that need to be fixed!');
        process.exit(1);
    } else {
        console.log('\n✅ All song files have valid chord mappings!');
    }
} else {
    console.log('Songs directory not found!');
}