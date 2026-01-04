use crate::hit_detection::{HitResult, HitStats};

/// Scoring system
#[derive(Debug, Clone)]
pub struct Scorer {
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub hits: u32,
    pub misses: u32,
    
    combo_multiplier: u32,
}

impl Scorer {
    pub fn new() -> Self {
        Self {
            score: 0,
            combo: 0,
            max_combo: 0,
            hits: 0,
            misses: 0,
            combo_multiplier: 1,
        }
    }

    /// Reset score
    pub fn reset(&mut self) {
        self.score = 0;
        self.combo = 0;
        self.max_combo = 0;
        self.hits = 0;
        self.misses = 0;
        self.combo_multiplier = 1;
    }

    /// Register a hit result
    pub fn register_hit(&mut self, result: &HitResult) {
        match result {
            HitResult::Hit { accuracy, .. } => {
                self.combo += 1;
                self.hits += 1;
                
                if self.combo > self.max_combo {
                    self.max_combo = self.combo;
                }

                // Update combo multiplier
                self.combo_multiplier = match self.combo {
                    0..=9 => 1,
                    10..=19 => 2,
                    20..=29 => 3,
                    _ => 4,
                };

                // Calculate points based on accuracy and multiplier
                let base_points = 100.0 * accuracy;
                let points = (base_points * self.combo_multiplier as f64) as u32;
                self.score += points;
            }
            HitResult::Miss { .. } => {
                self.combo = 0;
                self.combo_multiplier = 1;
                self.misses += 1;
            }
        }
    }

    /// Add sustain bonus points
    pub fn add_sustain_bonus(&mut self, points: u32) {
        self.score += points * self.combo_multiplier;
    }

    /// Get accuracy percentage
    pub fn get_accuracy(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        (self.hits as f64 / total as f64) * 100.0
    }

    /// Get grade based on accuracy
    pub fn get_grade(&self) -> Grade {
        let accuracy = self.get_accuracy();
        match accuracy {
            a if a >= 95.0 => Grade::S,
            a if a >= 90.0 => Grade::A,
            a if a >= 80.0 => Grade::B,
            a if a >= 70.0 => Grade::C,
            a if a >= 60.0 => Grade::D,
            _ => Grade::F,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
    F,
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::S => write!(f, "S"),
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

impl Default for Scorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hit_detection::{ChordEventHit, MissReason};

    #[test]
    fn test_scoring_basic() {
        let mut scorer = Scorer::new();

        let hit = HitResult::Hit {
            event: ChordEventHit {
                beat: 0.0,
                chord: "C".to_string(),
                is_sustain: false,
            },
            accuracy: 1.0,
        };

        scorer.register_hit(&hit);
        assert_eq!(scorer.hits, 1);
        assert_eq!(scorer.combo, 1);
        assert_eq!(scorer.score, 100);
    }

    #[test]
    fn test_combo_multiplier() {
        let mut scorer = Scorer::new();

        // Build up combo
        for _ in 0..15 {
            let hit = HitResult::Hit {
                event: ChordEventHit {
                    beat: 0.0,
                    chord: "C".to_string(),
                    is_sustain: false,
                },
                accuracy: 1.0,
            };
            scorer.register_hit(&hit);
        }

        // At combo 15, multiplier should be 2x
        assert_eq!(scorer.combo_multiplier, 2);
        assert_eq!(scorer.combo, 15);
    }

    #[test]
    fn test_accuracy() {
        let mut scorer = Scorer::new();

        // 3 hits, 1 miss = 75% accuracy
        for _ in 0..3 {
            scorer.register_hit(&HitResult::Hit {
                event: ChordEventHit {
                    beat: 0.0,
                    chord: "C".to_string(),
                    is_sustain: false,
                },
                accuracy: 1.0,
            });
        }

        scorer.register_hit(&HitResult::Miss {
            reason: MissReason::WrongFrets,
        });

        assert_eq!(scorer.get_accuracy(), 75.0);
        assert_eq!(scorer.get_grade(), Grade::C);
    }
}
