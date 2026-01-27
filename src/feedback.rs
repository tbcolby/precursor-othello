//! Haptic and audio feedback

/// Get a random bit using hardware TRNG
pub fn random_bit() -> bool {
    #[cfg(target_os = "none")]
    {
        let trng = trng::Trng::new(&xous_names::XousNames::new().unwrap()).ok();
        if let Some(t) = trng {
            return (t.get_u32().unwrap_or(0) & 1) != 0;
        }
    }
    // Fallback for hosted mode
    false
}

/// Vibrate for a move being played
pub fn vibrate_move() {
    #[cfg(target_os = "none")]
    {
        if let Ok(llio) = llio::Llio::new(&xous_names::XousNames::new().unwrap()) {
            llio.vibe(llio::VibePattern::Short).ok();
        }
    }
}

/// Vibrate for an invalid move attempt
pub fn vibrate_invalid() {
    #[cfg(target_os = "none")]
    {
        if let Ok(llio) = llio::Llio::new(&xous_names::XousNames::new().unwrap()) {
            llio.vibe(llio::VibePattern::Double).ok();
        }
    }
}

/// Vibrate for game over
pub fn vibrate_game_over() {
    #[cfg(target_os = "none")]
    {
        if let Ok(llio) = llio::Llio::new(&xous_names::XousNames::new().unwrap()) {
            llio.vibe(llio::VibePattern::Long).ok();
        }
    }
}
