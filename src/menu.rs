//! Menu system

/// Menu context (determines which items are shown)
#[derive(Debug, Clone, Copy)]
pub enum MenuContext {
    MainMenu { has_save: bool },
    Playing,
    GameOver,
    WhatIf,
}

/// Menu item actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Help,
    NewGame,
    Resume,
    Statistics,
    Settings,
    MoveHistory,
    Hint,
    Undo,
    Resign,
    SaveAndExit,
    WhatIf,
    ExitWhatIf,
    MainMenu,
}

impl MenuItem {
    /// Get the display label for this item
    pub fn label(&self) -> &'static str {
        match self {
            MenuItem::Help => "Help",
            MenuItem::NewGame => "New Game",
            MenuItem::Resume => "Resume Game",
            MenuItem::Statistics => "Statistics",
            MenuItem::Settings => "Settings",
            MenuItem::MoveHistory => "Move History",
            MenuItem::Hint => "Hint",
            MenuItem::Undo => "Undo",
            MenuItem::Resign => "Resign",
            MenuItem::SaveAndExit => "Save & Exit",
            MenuItem::WhatIf => "What If",
            MenuItem::ExitWhatIf => "Exit What If",
            MenuItem::MainMenu => "Main Menu",
        }
    }
}

/// Menu state
pub struct Menu {
    /// Whether the menu is visible
    pub visible: bool,
    /// Currently selected index
    pub selected: usize,
    /// Menu items
    pub items: Vec<MenuItem>,
}

impl Menu {
    /// Create a new menu
    pub fn new() -> Self {
        Self {
            visible: false,
            selected: 0,
            items: Vec::new(),
        }
    }

    /// Open the menu for a given context
    pub fn open(&mut self, context: MenuContext) {
        self.items = match context {
            MenuContext::MainMenu { has_save } => {
                let mut items = vec![
                    MenuItem::Help,
                    MenuItem::NewGame,
                ];
                if has_save {
                    items.push(MenuItem::Resume);
                }
                items.push(MenuItem::Statistics);
                items.push(MenuItem::Settings);
                items
            }
            MenuContext::Playing => {
                vec![
                    MenuItem::Help,
                    MenuItem::MoveHistory,
                    MenuItem::Hint,
                    MenuItem::Undo,
                    MenuItem::Resign,
                    MenuItem::SaveAndExit,
                    MenuItem::NewGame,
                ]
            }
            MenuContext::GameOver => {
                vec![
                    MenuItem::Help,
                    MenuItem::WhatIf,
                    MenuItem::MoveHistory,
                    MenuItem::NewGame,
                    MenuItem::MainMenu,
                ]
            }
            MenuContext::WhatIf => {
                vec![
                    MenuItem::Help,
                    MenuItem::ExitWhatIf,
                ]
            }
        };
        self.selected = 0;
        self.visible = true;
    }

    /// Close the menu
    pub fn close(&mut self) {
        self.visible = false;
    }

    /// Move selection up
    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn down(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Select current item
    pub fn select(&self) -> Option<MenuItem> {
        self.items.get(self.selected).copied()
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
