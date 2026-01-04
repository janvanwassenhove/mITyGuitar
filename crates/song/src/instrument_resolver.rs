use crate::chart::InstrumentRef;

/// Resolved instrument information
#[derive(Debug, Clone)]
pub struct ResolvedInstrument {
    pub instrument_type: String,
    pub label: String,
    pub is_available: bool,
    pub fallback_used: bool,
}

/// Instrument resolver
pub struct InstrumentResolver {
    available_instruments: Vec<(String, String)>, // (type, label) pairs
    global_default: (String, String),
}

impl InstrumentResolver {
    pub fn new(
        available_instruments: Vec<(String, String)>,
        global_default: (String, String),
    ) -> Self {
        Self {
            available_instruments,
            global_default,
        }
    }

    /// Resolve an instrument reference to an available instrument
    pub fn resolve(
        &self,
        default: &InstrumentRef,
        fallback: &InstrumentRef,
        user_override: Option<&InstrumentRef>,
    ) -> ResolvedInstrument {
        // Try user override first
        if let Some(override_ref) = user_override {
            if self.is_available(&override_ref.instrument_type, &override_ref.label) {
                return ResolvedInstrument {
                    instrument_type: override_ref.instrument_type.clone(),
                    label: override_ref.label.clone(),
                    is_available: true,
                    fallback_used: false,
                };
            }
        }

        // Try default instrument
        if self.is_available(&default.instrument_type, &default.label) {
            return ResolvedInstrument {
                instrument_type: default.instrument_type.clone(),
                label: default.label.clone(),
                is_available: true,
                fallback_used: false,
            };
        }

        // Try fallback instrument
        if self.is_available(&fallback.instrument_type, &fallback.label) {
            return ResolvedInstrument {
                instrument_type: fallback.instrument_type.clone(),
                label: fallback.label.clone(),
                is_available: true,
                fallback_used: true,
            };
        }

        // Use global default
        ResolvedInstrument {
            instrument_type: self.global_default.0.clone(),
            label: self.global_default.1.clone(),
            is_available: true,
            fallback_used: true,
        }
    }

    /// Check if an instrument is available
    fn is_available(&self, instrument_type: &str, label: &str) -> bool {
        self.available_instruments
            .iter()
            .any(|(t, l)| t == instrument_type && l == label)
    }

    /// Get list of available instruments
    pub fn get_available_instruments(&self) -> &[(String, String)] {
        &self.available_instruments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_instruments() -> Vec<(String, String)> {
        vec![
            ("soundfont".to_string(), "Clean Guitar".to_string()),
            ("soundfont".to_string(), "Distortion".to_string()),
            ("virtual".to_string(), "Basic Guitar".to_string()),
        ]
    }

    #[test]
    fn test_resolve_default() {
        let instruments = create_test_instruments();
        let resolver = InstrumentResolver::new(
            instruments,
            ("virtual".to_string(), "Basic Guitar".to_string()),
        );

        let default = InstrumentRef {
            instrument_type: "soundfont".to_string(),
            label: "Clean Guitar".to_string(),
        };

        let fallback = InstrumentRef {
            instrument_type: "virtual".to_string(),
            label: "Basic Guitar".to_string(),
        };

        let resolved = resolver.resolve(&default, &fallback, None);
        assert_eq!(resolved.label, "Clean Guitar");
        assert!(!resolved.fallback_used);
    }

    #[test]
    fn test_resolve_fallback() {
        let instruments = create_test_instruments();
        let resolver = InstrumentResolver::new(
            instruments,
            ("virtual".to_string(), "Basic Guitar".to_string()),
        );

        let default = InstrumentRef {
            instrument_type: "soundfont".to_string(),
            label: "Non Existent".to_string(),
        };

        let fallback = InstrumentRef {
            instrument_type: "virtual".to_string(),
            label: "Basic Guitar".to_string(),
        };

        let resolved = resolver.resolve(&default, &fallback, None);
        assert_eq!(resolved.label, "Basic Guitar");
        assert!(resolved.fallback_used);
    }

    #[test]
    fn test_resolve_user_override() {
        let instruments = create_test_instruments();
        let resolver = InstrumentResolver::new(
            instruments,
            ("virtual".to_string(), "Basic Guitar".to_string()),
        );

        let default = InstrumentRef {
            instrument_type: "soundfont".to_string(),
            label: "Clean Guitar".to_string(),
        };

        let fallback = InstrumentRef {
            instrument_type: "virtual".to_string(),
            label: "Basic Guitar".to_string(),
        };

        let override_inst = InstrumentRef {
            instrument_type: "soundfont".to_string(),
            label: "Distortion".to_string(),
        };

        let resolved = resolver.resolve(&default, &fallback, Some(&override_inst));
        assert_eq!(resolved.label, "Distortion");
        assert!(!resolved.fallback_used);
    }
}
