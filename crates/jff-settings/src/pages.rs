//! Everything the window shows, as plain data. widgets.rs and lists.rs turn it into rows.
//!
//! Paths, ranges and defaults were checked against src/config/mod.rs in the compositor.
//! New setting: add a Field here. New page: add a Page in build() and an icon in main.rs.
//!
//! Capitalization follows the GNOME HIG: Title Case for pages, groups and row titles,
//! sentence case (no final period) for subtitles.

pub enum Kind {
    /// Whole number in min..=max. `default` is used when the key is missing, `block_default`
    /// when the parent block exists but the key doesn't (`border { }` alone is a 2px border,
    /// no border block at all is none). `optional` adds a reset button, `positional` is for
    /// `parent N` written as `parent { child N }`.
    Int {
        min: i64,
        max: i64,
        default: i64,
        block_default: Option<i64>,
        optional: bool,
        positional: bool,
    },
    /// Decimal number. `digits` is how many decimals the row shows, `optional` adds a reset
    /// button that removes the key.
    Float {
        min: f64,
        max: f64,
        step: f64,
        digits: u32,
        default: f64,
        optional: bool,
    },
    /// Bare name is on, `#false` is off.
    Flag { default: bool },
    /// Switch for things that only exist by being written (`off`, `content-type-hint`).
    Presence,
    /// Unset, yes or no. `unset` is what the first entry says ("Any", "Default").
    Tri { unset: &'static str },
    /// `path` is a block that gets turned off by an `off` line inside it. `needs_block` is for
    /// blocks that only exist when they're on (blur).
    Enable { needs_block: bool },
    /// Pairs of (value in the file, label in the list).
    Choice {
        options: &'static [(&'static str, &'static str)],
        default: &'static str,
    },
    /// Like `Choice`, but the first entry (`unset`) removes the key, so the device or the
    /// compositor keeps its own default.
    Pick {
        options: &'static [(&'static str, &'static str)],
        unset: &'static str,
    },
    /// Free text, empty removes the key. Applied with Enter or the check button.
    Text,
    /// `mode "WIDTHxHEIGHT@HZ"` of a display: a searchable list of what the display offers
    /// (from `wlr-randr`) with Automatic first, or a text entry when there is no list.
    Mode,
    /// Written as "#rrggbbaa".
    Color {
        default: &'static str,
        optional: bool,
    },
    /// Two numbers on one line, like `position X Y` or `open-size W H`. Can be reset.
    Pair {
        min: i64,
        max: i64,
        default: (i64, i64),
    },
    /// Three decimal numbers on one line, like `hdr-white-balance R G B`. Can be reset.
    Triple {
        min: f64,
        max: f64,
        step: f64,
        default: [f64; 3],
    },
}

pub struct Field {
    /// Relative to the root the page builds under (empty for the plain pages).
    pub path: &'static [&'static str],
    /// If set, the value is this property of the node at `path`.
    pub prop: Option<&'static str>,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub kind: Kind,
}

/// A setting that's a list of lines, like `ignore-window "firefox"` or `autostart "foot"`.
#[derive(Clone)]
pub struct TextList {
    pub parent: &'static [&'static str],
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    /// Title of each entry row.
    pub item: &'static str,
    /// Title of the row that adds an entry.
    pub add: &'static str,
    /// One line is several words (a program plus arguments) instead of a single string.
    pub words: bool,
}

pub struct Group {
    pub title: &'static str,
    pub description: &'static str,
    pub fields: Vec<Field>,
    pub lists: Vec<TextList>,
    /// First field decides if the others can be edited (blur on or off, border width 0).
    pub gated: bool,
}

pub enum Body {
    Groups(Vec<Group>),
    /// The monitor list, followed by these groups.
    Outputs(Vec<Group>),
    Rules,
    Binds,
    /// Touchpad and mouse groups, then the list of single devices.
    Pointer,
    /// The `tiling-containers` switch, then the container rules.
    Containers,
}

pub struct Page {
    pub id: &'static str,
    pub title: &'static str,
    /// Extra words the sidebar search matches, on top of the titles in the page.
    pub keywords: &'static str,
    pub body: Body,
}

impl Page {
    /// Lowercase text the sidebar search looks through.
    pub fn search_text(&self) -> String {
        let mut text = format!("{} {}", self.title, self.keywords);
        let mut add_group = |g: &Group| {
            text.push(' ');
            text.push_str(g.title);
            for f in &g.fields {
                text.push(' ');
                text.push_str(f.title);
            }
            for l in &g.lists {
                text.push(' ');
                text.push_str(l.title);
            }
        };
        match &self.body {
            Body::Groups(groups) | Body::Outputs(groups) => groups.iter().for_each(&mut add_group),
            Body::Rules | Body::Binds | Body::Pointer | Body::Containers => {}
        }
        text.to_lowercase()
    }
}

pub fn build() -> Vec<Page> {
    vec![
        Page {
            id: "appearance",
            title: "Appearance",
            keywords: "gaps corners radius border color decorations title bar osd font maximize fill",
            body: Body::Groups(appearance()),
        },
        Page {
            id: "effects",
            title: "Effects",
            keywords: "blur animation fade zoom slide",
            body: Body::Groups(effects()),
        },
        Page {
            id: "multitasking",
            title: "Multitasking",
            keywords: "workspaces hot corner overview switcher alt tab wall drag maximize focus follows mouse warp zoom magnifier edge close kill unresponsive clipboard history",
            body: Body::Groups(multitasking()),
        },
        Page {
            id: "containers",
            title: "Containers",
            keywords: "tiling groups tile layout name match app id title tag remember hide behind",
            body: Body::Containers,
        },
        Page {
            id: "displays",
            title: "Displays",
            keywords: "monitor output resolution mode refresh rate hdr tone map bt2390 knee content metadata sdr white peak icc profile position vrr tearing color management transfer gamma contrast saturation white balance black level shadow gamut luminance",
            body: Body::Outputs(displays()),
        },
        Page {
            id: "keyboard",
            title: "Keyboard",
            keywords: "layout xkb variant repeat delay num lock hotkeys",
            body: Body::Groups(keyboard()),
        },
        Page {
            id: "pointer",
            title: "Mouse & Touchpad",
            keywords: "cursor theme size hide typing pointer input tap click scroll natural acceleration speed profile flat adaptive left handed middle emulation device libinput typing external",
            body: Body::Pointer,
        },
        Page {
            id: "shortcuts",
            title: "Keyboard Shortcuts",
            keywords: "key bindings binds hotkey action spawn",
            body: Body::Binds,
        },
        Page {
            id: "rules",
            title: "Window Rules",
            keywords: "match app id title tag dialog floating size workspace open",
            body: Body::Rules,
        },
        Page {
            id: "startup",
            title: "Startup Applications",
            keywords: "autostart programs commands shell login restore session apps",
            body: Body::Groups(startup()),
        },
        Page {
            id: "system",
            title: "System",
            keywords: "screenshots log file level folder clipboard history power button suspend gpu render device",
            body: Body::Groups(system()),
        },
    ]
}

// ---------------------------------------------------------------- builders

fn field(
    path: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    kind: Kind,
) -> Field {
    Field {
        path,
        prop: None,
        title,
        subtitle,
        kind,
    }
}

fn int(
    path: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    min: i64,
    max: i64,
    default: i64,
) -> Field {
    field(
        path,
        title,
        subtitle,
        Kind::Int {
            min,
            max,
            default,
            block_default: None,
            optional: false,
            positional: false,
        },
    )
}

fn float(
    path: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    (min, max, step, digits): (f64, f64, f64, u32),
    default: f64,
) -> Field {
    field(
        path,
        title,
        subtitle,
        Kind::Float {
            min,
            max,
            step,
            digits,
            default,
            optional: false,
        },
    )
}

fn pick(
    path: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    options: &'static [(&'static str, &'static str)],
    unset: &'static str,
) -> Field {
    field(path, title, subtitle, Kind::Pick { options, unset })
}

/// On, off, or left to the device.
fn tri(path: &'static [&'static str], title: &'static str, subtitle: &'static str) -> Field {
    field(
        path,
        title,
        subtitle,
        Kind::Tri {
            unset: "Device Default",
        },
    )
}

fn flag(
    path: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    default: bool,
) -> Field {
    field(path, title, subtitle, Kind::Flag { default })
}

fn text(path: &'static [&'static str], title: &'static str) -> Field {
    field(path, title, "", Kind::Text)
}

fn color(path: &'static [&'static str], title: &'static str, default: &'static str) -> Field {
    field(
        path,
        title,
        "",
        Kind::Color {
            default,
            optional: false,
        },
    )
}

fn enable(path: &'static [&'static str], title: &'static str, needs_block: bool) -> Field {
    field(path, title, "", Kind::Enable { needs_block })
}

fn choice(
    path: &'static [&'static str],
    title: &'static str,
    subtitle: &'static str,
    options: &'static [(&'static str, &'static str)],
    default: &'static str,
) -> Field {
    field(path, title, subtitle, Kind::Choice { options, default })
}

impl Field {
    fn block_default(mut self, value: i64) -> Self {
        if let Kind::Int { block_default, .. } = &mut self.kind {
            *block_default = Some(value);
        }
        self
    }

    fn optional(mut self) -> Self {
        match &mut self.kind {
            Kind::Int { optional, .. }
            | Kind::Float { optional, .. }
            | Kind::Color { optional, .. } => *optional = true,
            _ => {}
        }
        self
    }

    fn positional(mut self) -> Self {
        if let Kind::Int { positional, .. } = &mut self.kind {
            *positional = true;
        }
        self
    }

    fn prop(mut self, prop: &'static str) -> Self {
        self.prop = Some(prop);
        self
    }
}

fn group(title: &'static str, fields: Vec<Field>) -> Group {
    Group {
        title,
        description: "",
        fields,
        lists: Vec::new(),
        gated: false,
    }
}

impl Group {
    fn gated(mut self) -> Self {
        self.gated = true;
        self
    }

    fn described(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    fn lists(mut self, lists: Vec<TextList>) -> Self {
        self.lists = lists;
        self
    }
}

fn list(
    parent: &'static [&'static str],
    name: &'static str,
    title: &'static str,
    item: &'static str,
    add: &'static str,
) -> TextList {
    TextList {
        parent,
        name,
        title,
        description: "",
        item,
        add,
        words: false,
    }
}

// ------------------------------------------------------------------- pages

fn appearance() -> Vec<Group> {
    const ANIMATE: &[(&str, &str)] = &[
        ("none", "None"),
        ("spin", "Spin"),
        ("sweep", "Sweep"),
        ("smoke", "Smoke"),
        ("neon", "Neon"),
        ("aurora", "Aurora"),
    ];
    const LOOK: &[(&str, &str)] = &[("plain", "Plain"), ("paper", "Paper")];
    vec![
        group(
            "Windows",
            vec![
                int(
                    &["gaps"],
                    "Gaps",
                    "Space around maximized windows, in pixels",
                    0,
                    512,
                    0,
                ),
                int(
                    &["radius"],
                    "Corner Radius",
                    "In pixels, 0 for square corners",
                    0,
                    256,
                    0,
                ),
                int(
                    &["popup-radius"],
                    "Popup Corner Radius",
                    "Follows the corner radius when not set",
                    0,
                    256,
                    0,
                )
                .optional(),
                flag(
                    &["no-client-decorations"],
                    "Server-Side Decorations Only",
                    "Ask windows not to draw their own title bar",
                    false,
                ),
                flag(
                    &["maximize-fill"],
                    "Maximized Windows Fill the Screen",
                    "No gap, border or rounded corners around a maximized window. Off gives them back",
                    true,
                ),
            ],
        ),
        group(
            "Border",
            vec![
                int(
                    &["border", "width"],
                    "Border Width",
                    "In pixels, 0 turns the border off",
                    0,
                    64,
                    0,
                )
                .block_default(2),
                int(
                    &["border", "fade-ms"],
                    "Focus Fade",
                    "How long the color takes to change, in milliseconds",
                    0,
                    5_000,
                    150,
                ),
                choice(
                    &["border", "look"],
                    "Look",
                    "Paper draws the border as a sheet around the window",
                    LOOK,
                    "plain",
                ),
                color(&["border", "color"], "Color", "#595966ff"),
                color(&["border", "focus-color"], "Focused Color", "#598cf2ff"),
                field(
                    &["border", "focus-color-2"],
                    "Second Focused Color",
                    "Needed for an animated border",
                    Kind::Color {
                        default: "#598cf2ff",
                        optional: true,
                    },
                ),
                choice(&["border", "animate"], "Animation", "", ANIMATE, "none"),
                int(
                    &["border", "period-ms"],
                    "Animation Period",
                    "Time for one turn, in milliseconds",
                    0,
                    600_000,
                    3_000,
                ),
            ],
        )
        .gated(),
        group(
            "On-Screen Display",
            vec![
                enable(&["osd"], "On-Screen Display", false),
                int(
                    &["osd", "timeout-ms"],
                    "Timeout",
                    "In milliseconds, from 100 to 60000",
                    100,
                    60_000,
                    1_800,
                ),
                text(&["osd", "font"], "Font Family"),
            ],
        )
        .gated(),
    ]
}

fn effects() -> Vec<Group> {
    const OPEN: &[(&str, &str)] = &[
        ("fade", "Fade"),
        ("zoom", "Zoom"),
        ("slide", "Slide"),
        ("pop", "Pop"),
        ("none", "None"),
    ];
    vec![
        group(
            "Blur",
            vec![
                enable(&["blur"], "Blur", true),
                int(
                    &["blur", "passes"],
                    "Passes",
                    "More passes make a softer blur",
                    1,
                    31,
                    3,
                ),
                field(
                    &["blur", "offset"],
                    "Offset",
                    "",
                    Kind::Float {
                        min: 0.1,
                        max: 64.0,
                        step: 0.5,
                        digits: 1,
                        default: 3.0,
                        optional: false,
                    },
                ),
                flag(&["blur", "all-windows"], "Blur All Windows", "", false),
                flag(
                    &["blur", "all-layers"],
                    "Blur All Panels and Overlays",
                    "",
                    false,
                ),
                flag(
                    &["blur", "behind-windows"],
                    "Blur Behind Windows",
                    "",
                    false,
                ),
            ],
        )
        .gated()
        .lists(vec![
            list(
                &["blur"],
                "ignore-layer",
                "Panels Never Blurred",
                "Panel Namespace",
                "Add a Panel Namespace",
            ),
            list(
                &["blur"],
                "only-layer",
                "Only These Panels Are Blurred",
                "Panel Namespace",
                "Add a Panel Namespace",
            ),
            list(
                &["blur"],
                "ignore-window",
                "Apps Never Blurred",
                "App ID",
                "Add an App ID",
            ),
            list(
                &["blur"],
                "only-window",
                "Only These Apps Are Blurred",
                "App ID",
                "Add an App ID",
            ),
        ]),
        group(
            "Window Animations",
            vec![
                enable(&["window-animations"], "Window Animations", false),
                choice(
                    &["window-animations", "open"],
                    "Open Animation",
                    "",
                    OPEN,
                    "fade",
                ),
                int(
                    &["window-animations", "open-ms"],
                    "Open Duration",
                    "In milliseconds",
                    0,
                    10_000,
                    100,
                ),
                int(
                    &["window-animations", "close-ms"],
                    "Close Duration",
                    "In milliseconds",
                    0,
                    10_000,
                    80,
                ),
            ],
        )
        .gated(),
    ]
}

fn multitasking() -> Vec<Group> {
    const MODIFIERS: &[(&str, &str)] = &[
        ("Alt", "Alt"),
        ("Super", "Super"),
        ("Ctrl", "Ctrl"),
        ("Shift", "Shift"),
    ];
    vec![
        group(
            "Workspaces",
            vec![
                int(
                    &["workspaces", "count"],
                    "Number of Workspaces",
                    "",
                    1,
                    64,
                    9,
                )
                .positional(),
                flag(
                    &["workspaces", "per-output"],
                    "Workspaces on Each Display",
                    "Every monitor gets its own set",
                    false,
                ),
            ],
        ),
        group(
            "Focus",
            vec![
                flag(
                    &["focus-follows-mouse"],
                    "Focus Follows Mouse",
                    "The keyboard moves to the window under the pointer",
                    false,
                ),
                flag(
                    &["focus-new-windows"],
                    "Focus New Windows",
                    "Off stops a new window from taking the keyboard while you type in another one",
                    true,
                ),
                flag(
                    &["edge-warp"],
                    "Edge Warp",
                    "Pushing the pointer into the edge of a display with no neighbour there sends it to the next display",
                    false,
                ),
                flag(
                    &["warp-mouse-to-focus"],
                    "Mouse Follows Focus",
                    "The pointer jumps to a window that gets focus by other means, like Alt+Tab. Use it with Focus Follows Mouse",
                    false,
                ),
            ],
        ),
        group(
            "Windows",
            vec![
                flag(&["maximize-new-windows"], "Maximize New Windows", "", true),
                choice(
                    &["drag-modifier"],
                    "Drag Modifier",
                    "Modifier key for dragging windows",
                    MODIFIERS,
                    "Alt",
                ),
            ],
        ),
        group(
            "Hot Corner",
            vec![
                int(
                    &["hot-corner", "size"],
                    "Size",
                    "In pixels, 0 turns the hot corner off",
                    0,
                    64,
                    0,
                )
                .block_default(4)
                .positional(),
                int(
                    &["hot-corner", "dwell-ms"],
                    "Dwell Time",
                    "How long the pointer waits in the corner, in milliseconds",
                    0,
                    5_000,
                    250,
                ),
                int(
                    &["hot-corner", "touch-size"],
                    "Touch Corner Size",
                    "The corner a finger starts the swipe in, in pixels, 0 for no touch hot corner",
                    0,
                    200,
                    48,
                ),
            ],
        )
        .gated()
        .lists(vec![list(
            &["hot-corner"],
            "monitor",
            "Displays with a Hot Corner",
            "Display",
            "Add a Display Name",
        )]),
        group(
            "Window Switcher",
            vec![
                int(
                    &["switcher", "thumbnail-height"],
                    "Thumbnail Height",
                    "In pixels, jff picks a size when not set",
                    32,
                    4_096,
                    120,
                )
                .optional(),
                int(
                    &["switcher", "slide-ms"],
                    "Slide Duration",
                    "In milliseconds",
                    0,
                    2_000,
                    120,
                ),
                flag(
                    &["switcher", "group-containers"],
                    "Group Containers",
                    "A tiling container is one entry with all its windows in it",
                    true,
                ),
            ],
        ),
        group(
            "Overview",
            vec![
                flag(
                    &["overview", "auto-open"],
                    "Open on an Empty Display",
                    "Opens the overview when the display in use has no windows",
                    true,
                ),
                int(
                    &["overview", "animation-ms"],
                    "Animation Duration",
                    "How long it takes to open and close, in milliseconds, 0 for none",
                    0,
                    2_000,
                    320,
                ),
                flag(
                    &["overview", "blur"],
                    "Blur Frames and Search",
                    "Needs blur turned on",
                    false,
                ),
                int(
                    &["overview", "selection-width"],
                    "Selection Outline",
                    "In pixels, 0 for none",
                    0,
                    64,
                    4,
                ),
                color(
                    &["overview", "selection-color"],
                    "Selection Color",
                    "#598cf299",
                ),
                int(
                    &["overview", "border"],
                    "Workspace Border",
                    "Border around the workspace squares, in pixels, 0 for none",
                    0,
                    64,
                    2,
                ),
                field(
                    &["overview", "border-color"],
                    "Workspace Border Color",
                    "",
                    Kind::Color {
                        default: "#ebedf980",
                        optional: true,
                    },
                ),
                field(
                    &["overview", "backdrop-color"],
                    "Backdrop Color",
                    "Laid over the whole display behind the workspaces",
                    Kind::Color {
                        default: "#101018ff",
                        optional: true,
                    },
                ),
                field(
                    &["overview", "search-background-color"],
                    "Search Background",
                    "The search box and the grid of apps",
                    Kind::Color {
                        default: "#1c1c23ff",
                        optional: true,
                    },
                ),
                field(
                    &["overview", "search-font-color"],
                    "Search Text Color",
                    "",
                    Kind::Color {
                        default: "#f2f2f7ff",
                        optional: true,
                    },
                ),
            ],
        ),
        group(
            "Overview Containers",
            vec![
                color(
                    &["overview", "container-background-color"],
                    "Background",
                    "#0d0f1a66",
                ),
                int(
                    &["overview", "container-border-width"],
                    "Border Width",
                    "In pixels, 0 for none",
                    0,
                    64,
                    2,
                ),
                color(
                    &["overview", "container-border-color"],
                    "Border Color",
                    "#bfc7e099",
                ),
                color(
                    &["overview", "container-font-color"],
                    "Name Color",
                    "#f2f2f7ff",
                ),
                int(
                    &["overview", "container-radius"],
                    "Corner Radius",
                    "In pixels. Follows the corner radius when not set",
                    0,
                    256,
                    8,
                )
                .optional(),
            ],
        )
        .described("How a tiling container looks in the overview"),
        group(
            "Workspace Wall",
            vec![
                enable(&["workspace-wall"], "Workspace Wall", true),
                int(
                    &["workspace-wall", "height"],
                    "Height",
                    "In pixels",
                    32,
                    400,
                    96,
                )
                .block_default(96),
                flag(
                    &["workspace-wall", "thumbnails"],
                    "Show Thumbnails",
                    "",
                    true,
                ),
            ],
        )
        .gated(),
        group(
            "Zoom",
            vec![
                float(
                    &["zoom", "step"],
                    "Zoom Step",
                    "What one zoom-in multiplies the zoom by",
                    (1.01, 4.0, 0.05, 2),
                    1.2,
                ),
                float(
                    &["zoom", "max"],
                    "Maximum Zoom",
                    "The most that zooming in goes to",
                    (1.0, 64.0, 1.0, 1),
                    20.0,
                ),
                choice(
                    &["zoom", "tracking"],
                    "Tracking",
                    "How the part shown follows the pointer",
                    &[
                        ("proportional", "Proportional"),
                        ("centered", "Centered"),
                        ("push", "Push"),
                        ("disabled", "Off"),
                    ],
                    "proportional",
                ),
                choice(
                    &["zoom", "pointer"],
                    "Pointer",
                    "Scale magnifies it with the screen, Keep leaves it at its normal size",
                    &[("scale", "Scale"), ("keep", "Keep Size"), ("hide", "Hide")],
                    "scale",
                ),
                int(
                    &["zoom", "animation-ms"],
                    "Animation Duration",
                    "How long a change of the zoom takes, in milliseconds",
                    0,
                    2_000,
                    200,
                ),
                choice(
                    &["zoom", "scroll-modifier"],
                    "Scroll Modifier",
                    "Hold it and scroll to zoom",
                    &[
                        ("Super", "Super"),
                        ("Alt", "Alt"),
                        ("Ctrl", "Ctrl"),
                        ("Shift", "Shift"),
                        ("none", "Off"),
                    ],
                    "Super",
                ),
            ],
        ),
        group(
            "Unresponsive Windows",
            vec![
                flag(
                    &["close-kill", "enable"],
                    "Kill Windows That Don't Close",
                    "Sends SIGKILL to the program of a window still open after it was asked to close. Every other window of that program and anything unsaved is lost",
                    false,
                ),
                float(
                    &["close-kill", "seconds"],
                    "Wait Before Killing",
                    "In seconds",
                    (1.0, 3_600.0, 0.5, 1),
                    5.0,
                ),
            ],
        )
        .gated(),
    ]
}

/// Goes below the monitor list on the Displays page.
fn displays() -> Vec<Group> {
    vec![
        group(
            "Color",
            vec![
                flag(
                    &["color-management", "off"],
                    "Disable Color Management",
                    "Takes effect after a restart",
                    false,
                ),
                flag(
                    &["color-management", "convert-srgb"],
                    "Convert sRGB Windows Too",
                    "Stops oversaturation on wide-gamut monitors",
                    false,
                ),
                flag(
                    &["color-management", "fullscreen-passthrough"],
                    "Never Convert Fullscreen Windows",
                    "Keeps direct scanout working",
                    false,
                ),
                flag(
                    &["color-representation", "yuv"],
                    "YUV Color Representation",
                    "",
                    false,
                ),
            ],
        ),
        group(
            "Gaming",
            vec![
                flag(
                    &["allow-tearing"],
                    "Allow Tearing",
                    "Lower latency in fullscreen games, at the cost of tearing",
                    false,
                ),
                flag(
                    &["force-tearing"],
                    "Force Tearing",
                    "With Allow Tearing, every fullscreen window that covers a display tears, even if it never asked. Games under Wine don't ask",
                    false,
                ),
                flag(
                    &["wine-single-output"],
                    "Wine Sees One Display",
                    "A Wine program is shown only the display in use when it starts",
                    false,
                ),
            ],
        ),
    ]
}

fn keyboard() -> Vec<Group> {
    vec![
        group(
            "Layout",
            vec![
                text(&["keyboard", "xkb", "layout"], "Layout"),
                text(&["keyboard", "xkb", "variant"], "Variant"),
                text(&["keyboard", "xkb", "options"], "Options"),
            ],
        )
        .described("XKB names, like us, de or br for the layout, intl for the variant, caps:escape for options"),
        group(
            "Typing",
            vec![
                int(
                    &["keyboard", "repeat-delay"],
                    "Repeat Delay",
                    "Time before a held key repeats, in milliseconds",
                    0,
                    10_000,
                    200,
                ),
                int(
                    &["keyboard", "repeat-rate"],
                    "Repeat Rate",
                    "Repeats per second, 0 for no repeat",
                    0,
                    1_000,
                    25,
                ),
                flag(&["keyboard", "numlock"], "Num Lock on at Startup", "", true),
            ],
        ),
        group(
            "Application Hotkeys",
            vec![flag(
                &["global-hotkeys"],
                "Global Hotkeys for Apps",
                "Let apps bind their own hotkeys (xx-hotkey)",
                false,
            )],
        ),
        group(
            "Advanced",
            vec![
                text(&["keyboard", "xkb", "rules"], "Rules"),
                text(&["keyboard", "xkb", "model"], "Model"),
            ],
        ),
    ]
}

fn startup() -> Vec<Group> {
    vec![
        group(
            "Session",
            vec![flag(
                &["restore-apps"],
                "Restore Apps From the Last Session",
                "Starts the apps that were open when jff last ran and puts their windows back on their display and workspace. Read at startup",
                false,
            )],
        ),
        group("", Vec::new()).lists(vec![TextList {
            parent: &[],
            name: "autostart",
            title: "Programs",
            description: "Started with jff. Write the program first, then its arguments",
            item: "Program",
            add: "Add a Program and Its Arguments",
            words: true,
        }]),
        group("", Vec::new()).lists(vec![TextList {
            description: "Run through the shell, so pipes and variables work",
            ..list(
                &[],
                "sh-autostart",
                "Shell Commands",
                "Command",
                "Add a Command Line",
            )
        }]),
    ]
}

fn system() -> Vec<Group> {
    const LEVELS: &[(&str, &str)] = &[
        ("error", "Error"),
        ("warn", "Warning"),
        ("info", "Info"),
        ("debug", "Debug"),
        ("trace", "Trace"),
    ];
    vec![
        group(
            "Screenshots",
            vec![
                text(&["screenshot", "path"], "Screenshot Folder"),
                flag(
                    &["screenshot", "include-cursor"],
                    "Include the Pointer",
                    "",
                    false,
                ),
                flag(
                    &["screenshot", "copy-to-clipboard"],
                    "Copy to the Clipboard",
                    "",
                    true,
                ),
            ],
        ),
        group(
            "Clipboard",
            vec![
                enable(&["clipboard"], "Keep Copies After an App Closes", false),
                int(
                    &["clipboard", "history"],
                    "History Size",
                    "How many copies are kept, 1 is only the latest",
                    1,
                    500,
                    1,
                ),
                int(
                    &["clipboard", "max-size"],
                    "Largest Copy",
                    "In MiB. A bigger copy is not kept",
                    1,
                    256,
                    8,
                ),
            ],
        )
        .gated(),
        group(
            "Power",
            vec![
                choice(
                    &["power-key"],
                    "Power Button",
                    "System leaves it to logind. Needs systemd-inhibit and systemctl",
                    &[
                        ("suspend", "Suspend"),
                        ("power-off", "Power Off"),
                        ("ignore", "Do Nothing"),
                        ("system", "System Default"),
                    ],
                    "suspend",
                ),
                text(&["render-device"], "Graphics Card, Read at Startup"),
            ],
        )
        .described("The graphics card is a path like /dev/dri/card1, the primary one when empty"),
        group(
            "Logging",
            vec![
                flag(&["log", "enable"], "Write a Log File", "", false),
                text(&["log", "path"], "File or Folder"),
                choice(&["log", "level"], "Level", "", LEVELS, "info"),
            ],
        )
        .described("A path that ends with / is a folder")
        .gated(),
    ]
}

/// Rows for one `output "NAME" { }`, relative to that node.
pub fn output_fields() -> Vec<Field> {
    const SYNC: &[(&str, &str)] = &[
        ("off", "Off"),
        ("fullscreen", "Fullscreen Only"),
        ("always", "Always"),
    ];
    const TONE_MAP: &[(&str, &str)] = &[("knee", "Knee"), ("bt2390", "BT.2390")];
    const TRANSFORM: &[(&str, &str)] = &[
        ("90", "Rotate 90° Counter-Clockwise"),
        ("180", "Rotate 180°"),
        ("270", "Rotate 270° Counter-Clockwise"),
        ("flipped", "Flip"),
        ("flipped-90", "Flip and Rotate 90°"),
        ("flipped-180", "Flip and Rotate 180°"),
        ("flipped-270", "Flip and Rotate 270°"),
    ];
    const SDR_TRANSFER: &[(&str, &str)] = &[("srgb", "sRGB"), ("gamma22", "Gamma 2.2")];
    const GAMUT_MAP: &[(&str, &str)] = &[("compress", "Compress"), ("clip", "Clip")];
    vec![
        field(&["off"], "Turn Off This Display", "", Kind::Presence),
        field(
            &["position"],
            "Position",
            "Top-left corner on the shared desktop",
            Kind::Pair {
                min: -100_000,
                max: 100_000,
                default: (0, 0),
            },
        ),
        field(
            &["mode"],
            "Mode",
            "Read from wlr-randr. Automatic is the display's preferred mode",
            Kind::Mode,
        ),
        field(
            &["scale"],
            "Scale",
            "Draws the desktop larger, from 0.25 to 10",
            Kind::Float {
                min: 0.25,
                max: 10.0,
                step: 0.25,
                digits: 2,
                default: 1.0,
                optional: true,
            },
        ),
        pick(
            &["split"],
            "Container Split",
            "How a window that joins a tiling container here is split off. Needs tiling containers",
            &[
                ("auto", "Automatic"),
                ("horizontal", "Side by Side"),
                ("vertical", "One Above the Other"),
            ],
            "Default",
        ),
        pick(
            &["transform"],
            "Rotation",
            "Turns the picture onto the panel, for a display that is mounted sideways or upside down",
            TRANSFORM,
            "Normal",
        ),
        choice(
            &["adaptive-sync"],
            "Variable Refresh Rate",
            "",
            SYNC,
            "fullscreen",
        ),
        flag(
            &["hdr"],
            "HDR",
            "Signals HDR10 when the monitor and connector can do it, and stays SDR when they can't",
            false,
        ),
        int(
            &["sdr-white"],
            "SDR White",
            "Brightness of SDR white on an HDR display, in cd/m². Lower it for more room for highlights",
            10,
            1000,
            203,
        )
        .optional(),
        choice(
            &["sdr-transfer"],
            "SDR Decoding",
            "How SDR windows are read on an HDR display. Gamma 2.2 gives the darker darks most SDR displays show. Only used with HDR on",
            SDR_TRANSFER,
            "srgb",
        ),
        int(
            &["hdr-peak"],
            "HDR Peak",
            "Peak brightness in cd/m², used instead of the one the display reports",
            100,
            10_000,
            400,
        )
        .optional(),
        float(
            &["hdr-min-luminance"],
            "Display Black Level",
            "Black of the panel in cd/m², used instead of the one the display reports. The shadows are raised off it. Only used with HDR on",
            (0.0, 1.0, 0.01, 3),
            0.0,
        )
        .optional(),
        float(
            &["hdr-contrast"],
            "Contrast",
            "Around SDR white. Above 1 darks go down and highlights go up, under 1 the picture is flatter. Only used with HDR on",
            (0.5, 2.0, 0.05, 2),
            1.0,
        )
        .optional(),
        float(
            &["hdr-black-level"],
            "Shadow Lift",
            "Light added to the shadows in cd/m². A negative value crushes them. Only used with HDR on",
            (-5.0, 5.0, 0.1, 2),
            0.0,
        )
        .optional(),
        float(
            &["hdr-saturation"],
            "Saturation",
            "Color intensity, brightness is kept. 0 is gray. Only used with HDR on",
            (0.0, 2.0, 0.05, 2),
            1.0,
        )
        .optional(),
        field(
            &["hdr-white-balance"],
            "White Balance",
            "Gain of red, green and blue. Lower blue for a warmer picture. Only used with HDR on",
            Kind::Triple {
                min: 0.5,
                max: 1.5,
                step: 0.01,
                default: [1.0, 1.0, 1.0],
            },
        ),
        choice(
            &["hdr-tone-map"],
            "HDR Tone Mapping",
            "How highlights brighter than the display can show are rolled off. Only used with HDR on",
            TONE_MAP,
            "knee",
        ),
        flag(
            &["hdr-content-metadata"],
            "Use Content Brightness",
            "Fits the roll-off to the peak the content states. Only used with HDR on",
            true,
        ),
        choice(
            &["hdr-gamut-map"],
            "Out-of-Gamut Colors",
            "Compress moves colors the display can't show toward gray, Clip cuts each channel. Needs the display's primaries. Only used with HDR on",
            GAMUT_MAP,
            "compress",
        ),
        text(&["icc-profile"], "ICC Profile Path"),
        field(
            &["content-type-hint"],
            "Game Content Type",
            "Tells the monitor it's showing a game in fullscreen games",
            Kind::Presence,
        ),
        flag(&["color-management"], "Color Managed", "", true),
    ]
}

/// Rows for one `window-rule { }`, relative to that node. Only the first `match` line gets
/// edited, a rule with several keeps the rest untouched.
pub fn rule_fields() -> Vec<Field> {
    vec![
        field(&["match"], "Match App ID", "", Kind::Text).prop("app-id"),
        field(&["match"], "Match Title", "", Kind::Text).prop("title"),
        field(&["match"], "Match Tag", "", Kind::Text).prop("tag"),
        field(&["match"], "Match Dialogs", "", Kind::Tri { unset: "Any" }).prop("dialog"),
        field(
            &["match"],
            "Match Restored Windows",
            "",
            Kind::Tri { unset: "Any" },
        )
        .prop("restored"),
        text(&["open-on-output"], "Open on Display, a name or pointer"),
        int(&["open-workspace"], "Open on Workspace", "", 1, 64, 1).optional(),
        field(
            &["open-maximized"],
            "Open Maximized",
            "",
            Kind::Tri { unset: "Default" },
        ),
        field(
            &["open-size"],
            "Open Size",
            "Width and height",
            Kind::Pair {
                min: 1,
                max: 16_384,
                default: (800, 600),
            },
        ),
        field(
            &["open-position"],
            "Open Position",
            "Top-left corner from the top left of its display. The window floats",
            Kind::Pair {
                min: 0,
                max: 16_384,
                default: (0, 0),
            },
        ),
        flag(
            &["open-centered"],
            "Open Centered",
            "The middle of its display. The window floats",
            false,
        ),
        float(
            &["opacity"],
            "Opacity",
            "How opaque the window is drawn, from 0 (invisible) to 1. Set when the window opens",
            (0.0, 1.0, 0.05, 2),
            1.0,
        )
        .optional(),
        text(&["tag"], "Give the Window a Tag"),
        text(
            &["open-in-container"],
            "Open in Container, Needs Tiling Containers",
        ),
        int(
            &["sdr-white"],
            "SDR White",
            "Brightness of this window's SDR white on an HDR display, in cd/m², instead of the display's",
            10,
            1000,
            300,
        )
        .optional(),
        field(
            &["sdr"],
            "Read as sRGB",
            "Shows the window as plain sRGB whatever its client tagged it. Only used on an HDR display",
            Kind::Tri { unset: "Default" },
        ),
        flag(
            &["floating"],
            "Floating",
            "Keep the window at its own size",
            false,
        ),
        flag(
            &["always-on-top"],
            "Always on Top",
            "Stays above the other windows, even a fullscreen one",
            false,
        ),
        flag(
            &["sticky"],
            "On Every Workspace",
            "Shown on every workspace of its display",
            false,
        ),
        pick(
            &["content-type"],
            "Content Type",
            "What the window shows, whatever its client says",
            &[
                ("game", "Game"),
                ("video", "Video"),
                ("photo", "Photo"),
                ("none", "Ignore the Client"),
            ],
            "Default",
        ),
    ]
}

/// Which block of `input { }` the rows are for.
#[derive(Clone, Copy, PartialEq)]
pub enum DeviceBlock {
    Touchpad,
    Mouse,
    /// One `device "NAME"`: everything.
    Device,
}

/// Rows for `input { touchpad }`, `input { mouse }` or one `input { device "NAME" }`,
/// relative to that node. A row left on Device Default removes its line.
pub fn device_fields(block: DeviceBlock) -> Vec<Field> {
    const TAP_MAP: &[(&str, &str)] = &[
        (
            "left-right-middle",
            "One Finger Left, Two Right, Three Middle",
        ),
        (
            "left-middle-right",
            "One Finger Left, Two Middle, Three Right",
        ),
    ];
    const ACCEL: &[(&str, &str)] = &[("adaptive", "Adaptive"), ("flat", "Flat")];
    const SCROLL: &[(&str, &str)] = &[
        ("two-finger", "Two Fingers"),
        ("edge", "Edge"),
        ("on-button-down", "While Holding a Button"),
        ("none", "Off"),
    ];
    const SCROLL_BUTTON: &[(&str, &str)] = &[
        ("left", "Left"),
        ("right", "Right"),
        ("middle", "Middle"),
        ("side", "Side"),
        ("extra", "Extra"),
        ("forward", "Forward"),
        ("back", "Back"),
    ];
    const CLICK: &[(&str, &str)] = &[
        ("button-areas", "Button Areas"),
        ("clickfinger", "Number of Fingers"),
    ];
    let touchpad = block != DeviceBlock::Mouse;
    let mut rows = Vec::new();
    if touchpad {
        rows.push(tri(
            &["tap"],
            "Tap to Click",
            "On when not set, for a touchpad",
        ));
        rows.push(pick(
            &["tap-button-map"],
            "Tap Buttons",
            "Which button one, two and three finger taps press",
            TAP_MAP,
            "Device Default",
        ));
        rows.push(tri(
            &["drag"],
            "Tap and Drag",
            "Tap, then touch and move to drag",
        ));
        rows.push(tri(
            &["disable-while-typing"],
            "Disable While Typing",
            "Ignore touches while typing",
        ));
        rows.push(pick(
            &["click-method"],
            "Click Method",
            "How a press on the pad becomes a left, right or middle click",
            CLICK,
            "Device Default",
        ));
        rows.push(tri(
            &["disable-on-external-mouse"],
            "Disable With an External Mouse",
            "Send nothing while a mouse is plugged in",
        ));
    }
    rows.push(tri(
        &["natural-scroll"],
        "Natural Scrolling",
        "Content follows the fingers",
    ));
    rows.push(
        float(
            &["accel-speed"],
            "Pointer Speed",
            "From -1 (slowest) to 1 (fastest)",
            (-1.0, 1.0, 0.1, 2),
            0.0,
        )
        .optional(),
    );
    rows.push(pick(
        &["accel-profile"],
        "Acceleration",
        "Flat moves the pointer the same distance whatever the speed",
        ACCEL,
        "Device Default",
    ));
    rows.push(pick(
        &["scroll-method"],
        "Scroll Method",
        "",
        SCROLL,
        "Device Default",
    ));
    rows.push(pick(
        &["scroll-button"],
        "Scroll Button",
        "The button held for scrolling with While Holding a Button. A numeric code written by hand is kept until this is changed",
        SCROLL_BUTTON,
        "Device Default",
    ));
    rows.push(tri(
        &["middle-emulation"],
        "Middle Click Emulation",
        "Both buttons together are a middle click",
    ));
    rows.push(tri(
        &["left-handed"],
        "Left-Handed",
        "Swap the left and right buttons",
    ));
    rows.push(tri(&["off"], "Turn Off This Device", "Send nothing at all"));
    rows
}

pub fn touchpad_groups() -> Vec<Group> {
    vec![Group {
        title: "Touchpad",
        description: "Applies to every touchpad. A row on Device Default keeps what the touchpad does on its own",
        fields: device_fields(DeviceBlock::Touchpad),
        lists: Vec::new(),
        gated: false,
    }]
}

pub fn mouse_groups() -> Vec<Group> {
    vec![Group {
        title: "Mouse",
        description: "Applies to every other pointer device: mice, trackballs and pointing sticks",
        fields: device_fields(DeviceBlock::Mouse),
        lists: Vec::new(),
        gated: false,
    }]
}

/// The `tiling-containers` switch.
pub fn container_groups() -> Vec<Group> {
    vec![group(
        "Tiling Containers",
        vec![flag(
            &["tiling-containers"],
            "Tiling Containers",
            "Groups of windows that tile among themselves. Needed by container rules, Open in Container and the container shortcuts",
            false,
        ),
        flag(
            &["remember-container-layouts"],
            "Remember Layouts",
            "Windows tiled together again get the arrangement they had, until jff ends",
            false,
        ),
        flag(
            &["hide-behind-containers"],
            "Hide Windows Behind a Container",
            "While a container has the keyboard, the gaps between its tiles show the wallpaper",
            true,
        )],
    )]
}

/// Rows for one `container-rule { }`, relative to that node.
pub fn container_rule_fields() -> Vec<Field> {
    vec![
        field(&["match"], "Match App ID", "", Kind::Text).prop("app-id"),
        field(&["match"], "Match Title", "", Kind::Text).prop("title"),
        field(&["match"], "Match Tag", "", Kind::Text).prop("tag"),
        text(&["name"], "Container Name"),
        text(&["layout"], "Saved Layout to Apply"),
    ]
}

/// Rows for the cursor.
pub fn cursor_groups() -> Vec<Group> {
    vec![group(
        "Cursor",
        vec![
            text(&["cursor", "theme"], "Theme"),
            int(
                &["cursor", "size"],
                "Size",
                "In pixels. Falls back to $XCURSOR_SIZE, then 24. An app that is already open keeps the size it read at its start",
                1,
                512,
                24,
            )
            .optional(),
            flag(
                &["cursor", "hide-when-typing"],
                "Hide While Typing",
                "Back when the pointer moves",
                false,
            ),
            int(
                &["cursor", "hide-after-ms"],
                "Hide After Resting",
                "Milliseconds the pointer rests before it hides, 0 for never",
                0,
                600_000,
                0,
            ),
            int(
                &["cursor", "hide-after-idle"],
                "Hide While Fullscreen",
                "Seconds the pointer rests before it hides, only while the focused window is fullscreen. 0 for never",
                0,
                3_600,
                0,
            ),
        ],
    )
    .described("The theme is an XCursor folder name from ~/.icons or /usr/share/icons, and falls back to $XCURSOR_THEME, then default")]
}
