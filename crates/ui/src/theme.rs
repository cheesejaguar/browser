//! Theme and colors.

/// Color in RGBA format.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
}

/// UI theme.
#[derive(Clone, Debug)]
pub struct UiTheme {
    pub name: String,
    pub is_dark: bool,
    pub colors: ThemeColors,
}

/// Theme colors.
#[derive(Clone, Debug)]
pub struct ThemeColors {
    pub background: Color,
    pub foreground: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub border: Color,
    pub tab_background: Color,
    pub tab_active_background: Color,
    pub toolbar_background: Color,
    pub address_bar_background: Color,
}

impl UiTheme {
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            is_dark: false,
            colors: ThemeColors {
                background: Color::rgb(255, 255, 255),
                foreground: Color::rgb(33, 33, 33),
                primary: Color::rgb(66, 133, 244),
                secondary: Color::rgb(95, 99, 104),
                accent: Color::rgb(66, 133, 244),
                error: Color::rgb(234, 67, 53),
                warning: Color::rgb(251, 188, 4),
                success: Color::rgb(52, 168, 83),
                border: Color::rgb(218, 220, 224),
                tab_background: Color::rgb(241, 243, 244),
                tab_active_background: Color::rgb(255, 255, 255),
                toolbar_background: Color::rgb(241, 243, 244),
                address_bar_background: Color::rgb(255, 255, 255),
            },
        }
    }

    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            is_dark: true,
            colors: ThemeColors {
                background: Color::rgb(32, 33, 36),
                foreground: Color::rgb(232, 234, 237),
                primary: Color::rgb(138, 180, 248),
                secondary: Color::rgb(154, 160, 166),
                accent: Color::rgb(138, 180, 248),
                error: Color::rgb(242, 139, 130),
                warning: Color::rgb(253, 214, 99),
                success: Color::rgb(129, 201, 149),
                border: Color::rgb(60, 64, 67),
                tab_background: Color::rgb(41, 42, 45),
                tab_active_background: Color::rgb(53, 54, 58),
                toolbar_background: Color::rgb(41, 42, 45),
                address_bar_background: Color::rgb(53, 54, 58),
            },
        }
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self::light()
    }
}
