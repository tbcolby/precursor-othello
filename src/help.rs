//! Help screen content

use gam::{Gam, GlyphStyle};
use gam::menu::{Point, Rectangle, TextView, TextBounds};

use crate::app::OthelloApp;

/// Help context determines which help text is shown
#[derive(Debug, Clone, Copy)]
pub enum HelpContext {
    MainMenu,
    Playing,
    WhatIf,
}

/// Draw help screen
pub fn draw_help(app: &OthelloApp, gam: &Gam, context: HelpContext) {
    let gid = app.gid;

    // Title
    let title = match context {
        HelpContext::MainMenu => "OTHELLO v1.0",
        HelpContext::Playing => "OTHELLO - Playing",
        HelpContext::WhatIf => "OTHELLO - What If",
    };

    // Border
    let margin = 20isize;
    gam.draw_rectangle(
        gid,
        gam::menu::Rectangle::new_with_style(
            Point::new(margin, margin),
            Point::new(app.screensize.x - margin, app.screensize.y - margin),
            gam::menu::DrawStyle::new(
                gam::menu::PixelColor::Dark,
                gam::menu::PixelColor::Light,
                2,
            ),
        ),
    )
    .ok();

    // Title
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(
            margin,
            margin + 10,
            app.screensize.x - margin,
            margin + 40,
        )),
    );
    tv.style = GlyphStyle::Bold;
    use core::fmt::Write;
    write!(tv.text, "{}", title).ok();
    gam.post_textview(&mut tv).ok();

    // Content based on context
    let content = match context {
        HelpContext::MainMenu => HELP_MAIN_MENU,
        HelpContext::Playing => HELP_PLAYING,
        HelpContext::WhatIf => HELP_WHAT_IF,
    };

    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(
            margin + 10,
            margin + 50,
            app.screensize.x - margin - 10,
            app.screensize.y - margin - 40,
        )),
    );
    tv.style = GlyphStyle::Regular;
    write!(tv.text, "{}", content).ok();
    gam.post_textview(&mut tv).ok();

    // Footer
    let mut tv = TextView::new(
        gid,
        TextBounds::BoundingBox(Rectangle::new_coords(
            margin,
            app.screensize.y - margin - 30,
            app.screensize.x - margin,
            app.screensize.y - margin - 10,
        )),
    );
    tv.style = GlyphStyle::Small;
    write!(tv.text, "Press any key to close").ok();
    gam.post_textview(&mut tv).ok();
}

const HELP_MAIN_MENU: &str = r"Classic Reversi strategy game.
Outflank your opponent's discs
to flip them to your color.

Controls:

F1        Menu
F4        Exit
Up/Down   Navigate
Enter     Select

The player with the most discs
when the board is full wins!

In vs CPU mode, your color is
randomly assigned each game.";

const HELP_PLAYING: &str = r"Controls:

F1        Menu
F4        Save & Exit
F2        Show Hint

Arrows    Move cursor
Enter     Place disc
H         Toggle hints
U         Undo last move

Legend:
[=]  Your cursor
 *   Valid move
 #   Last move played";

const HELP_WHAT_IF: &str = r"Review and explore alternate
moves from any point in the game.

Controls:

F1        Menu
F4        Exit What If

Left/Right  Step back/forward
Home        Jump to start
End         Jump to end

Arrows      Move cursor
Enter       Play alternate move
            (branches the game)

Once you branch, continue playing
to explore 'what if' scenarios.";
