/// Display modes for the LCD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    Normal,
    Dashboard,
    Banner,
}

/// The curated set of semantic icons that LifelineTTY understands.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Icon {
    Battery,
    Heart,
    Wifi,
    Arrow,
    Bell,
    Note,
    Clockface,
    Duck,
    Check,
    Cross,
    Smile,
    OpenHeart,
    UpArrow,
    UpArrowRight,
    UpArrowLeft,
    DownArrow,
    DownArrowRight,
    DownArrowLeft,
    ReturnArrow,
    Hourglass,
    DegreeSymbol,
    DegreeC,
    DegreeF,
}

impl Icon {
    fn normalize(name: &str) -> String {
        name.trim()
            .to_ascii_lowercase()
            .replace('-', "_")
            .replace(' ', "_")
    }

    pub fn from_str(name: &str) -> Option<Self> {
        match Self::normalize(name).as_str() {
            "battery" => Some(Icon::Battery),
            "heart" | "heartbeat" => Some(Icon::Heart),
            "wifi" | "wlan" => Some(Icon::Wifi),
            "arrow" => Some(Icon::Arrow),
            "bell" => Some(Icon::Bell),
            "note" => Some(Icon::Note),
            "clockface" => Some(Icon::Clockface),
            "duck" => Some(Icon::Duck),
            "check" => Some(Icon::Check),
            "cross" => Some(Icon::Cross),
            "smile" => Some(Icon::Smile),
            "open_heart" | "openheart" => Some(Icon::OpenHeart),
            "up_arrow" => Some(Icon::UpArrow),
            "up_arrow_right" => Some(Icon::UpArrowRight),
            "up_arrow_left" => Some(Icon::UpArrowLeft),
            "down_arrow" => Some(Icon::DownArrow),
            "down_arrow_right" => Some(Icon::DownArrowRight),
            "down_arrow_left" => Some(Icon::DownArrowLeft),
            "return_arrow" | "return" => Some(Icon::ReturnArrow),
            "hourglass" => Some(Icon::Hourglass),
            "degree_symbol" | "degree" => Some(Icon::DegreeSymbol),
            "degree_c" | "degc" => Some(Icon::DegreeC),
            "degree_f" | "degf" => Some(Icon::DegreeF),
            _ => None,
        }
    }

    pub fn bitmap(&self) -> Option<[u8; 8]> {
        match self {
            Icon::Battery => Some([0x1f, 0x1f, 0x11, 0x11, 0x11, 0x11, 0x1f, 0x1f]),
            Icon::Heart => Some([0x00, 0x0a, 0x1f, 0x1f, 0x0e, 0x04, 0x00, 0x00]),
            Icon::Wifi => Some([0x00, 0x04, 0x0e, 0x15, 0x04, 0x1f, 0x04, 0x00]),
            Icon::Arrow | Icon::DownArrow => Some([0x00, 0x0e, 0x0e, 0x0e, 0x1f, 0x0e, 0x04, 0x00]),
            Icon::Bell => Some([0x04, 0x0e, 0x0e, 0x0e, 0x1f, 0x00, 0x04, 0x00]),
            Icon::Note => Some([0x02, 0x03, 0x02, 0x0e, 0x1e, 0x0c, 0x00, 0x00]),
            Icon::Clockface => Some([0x00, 0x0e, 0x15, 0x17, 0x11, 0x0e, 0x00, 0x00]),
            Icon::Duck => Some([0x00, 0x0c, 0x1d, 0x0f, 0x0f, 0x06, 0x00, 0x00]),
            Icon::Check => Some([0x00, 0x01, 0x03, 0x16, 0x1c, 0x08, 0x00, 0x00]),
            Icon::Cross => Some([0x00, 0x1b, 0x0e, 0x04, 0x0e, 0x1b, 0x00, 0x00]),
            Icon::Smile => Some([0x00, 0x0a, 0x0a, 0x00, 0x00, 0x11, 0x0e, 0x00]),
            Icon::OpenHeart => Some([0x00, 0x0a, 0x15, 0x11, 0x0a, 0x04, 0x00, 0x00]),
            Icon::UpArrow => Some([0x04, 0x0e, 0x1f, 0x0e, 0x0e, 0x0e, 0x00, 0x00]),
            Icon::UpArrowRight => Some([0x00, 0x0f, 0x03, 0x05, 0x09, 0x10, 0x00, 0x00]),
            Icon::UpArrowLeft => Some([0x00, 0x1e, 0x18, 0x14, 0x12, 0x01, 0x00, 0x00]),
            Icon::DownArrowRight => Some([0x00, 0x10, 0x09, 0x05, 0x03, 0x0f, 0x00, 0x00]),
            Icon::DownArrowLeft => Some([0x00, 0x01, 0x12, 0x14, 0x18, 0x1e, 0x00, 0x00]),
            Icon::ReturnArrow => Some([0x01, 0x01, 0x05, 0x09, 0x1f, 0x08, 0x04, 0x00]),
            Icon::Hourglass => Some([0x1f, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x1f, 0x00]),
            Icon::DegreeSymbol => Some([0x06, 0x09, 0x09, 0x06, 0x00, 0x00, 0x00, 0x00]),
            Icon::DegreeC => Some([0x18, 0x18, 0x03, 0x04, 0x04, 0x04, 0x03, 0x00]),
            Icon::DegreeF => Some([0x18, 0x18, 0x07, 0x04, 0x07, 0x04, 0x04, 0x00]),
        }
    }

    // ASCII fallbacks have been removed — missing glyphs should be handled by the
    // renderer or caller instead of silently substituting characters.
}

impl DisplayMode {
    pub(crate) fn parse(raw: Option<String>) -> Self {
        match raw.as_deref() {
            Some("dashboard") => DisplayMode::Dashboard,
            Some("banner") => DisplayMode::Banner,
            _ => DisplayMode::Normal,
        }
    }
}

pub(crate) fn parse_icons(raw: Option<Vec<String>>) -> Vec<Icon> {
    raw.unwrap_or_default()
        .into_iter()
        .filter_map(|name| Icon::from_str(&name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{DisplayMode, Icon};

    #[test]
    fn parses_display_mode_variants() {
        assert_eq!(
            DisplayMode::parse(Some("dashboard".into())),
            DisplayMode::Dashboard
        );
        assert_eq!(
            DisplayMode::parse(Some("banner".into())),
            DisplayMode::Banner
        );
        assert_eq!(
            DisplayMode::parse(Some("unknown".into())),
            DisplayMode::Normal
        );
    }

    #[test]
    fn icon_from_str_handles_variants() {
        assert_eq!(Icon::from_str("WiFi"), Some(Icon::Wifi));
        assert_eq!(Icon::from_str("open-heart"), Some(Icon::OpenHeart));
        assert_eq!(Icon::from_str("Up_Arrow"), Some(Icon::UpArrow));
        assert_eq!(Icon::from_str("degree_f"), Some(Icon::DegreeF));
    }

    #[test]
    fn icon_bitmap_matches_reference() {
        assert_eq!(
            Icon::Bell.bitmap(),
            Some([0x04, 0x0e, 0x0e, 0x0e, 0x1f, 0x00, 0x04, 0x00])
        );
        assert_eq!(
            Icon::Heart.bitmap(),
            Some([0x00, 0x0a, 0x1f, 0x1f, 0x0e, 0x04, 0x00, 0x00])
        );
    }

    // duplicate test removed; kept the canonical `icon_bitmap_matches_reference` above
}
