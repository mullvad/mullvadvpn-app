//! Since the old settings concepts do not map cleanly onto the new settings, this migration performs breaking changes to user settings. But it tries really hard not to!
//!
//! The migration has to consider the following scenarios:
//!
//! | Scenario | Multihop | DAITA | Direct Only | Magic Multihop | Filters |
//! |----------|----------|-------|-------------|----------------|---------|
//! |    1A    |    ❌    |   ❌  |      ❔     |       ❔       |    ❌   |
//! |    1B    |    ❌    |   ❌  |      ❔     |       ❔       |    ✅   |
//! |    2     |    ❌    |   ✅  |      ❌     |       ❔       |    ❌   |
//! |    3A    |    ❌    |   ✅  |      ❌     |       ❌       |    ✅   |
//! |    3B    |    ❌    |   ✅  |      ❌     |       ✅       |    ✅   |
//! |    4A    |    ❌    |   ✅  |      ✅     |       ❔       |    ❌   |
//! |    4B    |    ❌    |   ✅  |      ✅     |       ❔       |    ✅   |
//! |    5A    |    ✅    |   ❌  |      ❔     |       ❔       |    ❌   |
//! |    5B    |    ✅    |   ❌  |      ❔     |       ❔       |    ✅   |
//! |    6A    |    ✅    |   ✅  |      ❌     |       ❔       |    ❌   |
//! |    6B    |    ✅    |   ✅  |      ❌     |       ❔       |    ✅   |
//! |    7A    |    ✅    |   ✅  |      ✅     |       ❔       |    ❌   |
//! |    7B    |    ✅    |   ✅  |      ✅     |       ❔       |    ✅   |
//!
//! Filters are considered because they preivously affected DAITA through 'automatic multihop'. This
//! was undefined behaviour, but it was decided when this migration was architected to respect the
//! previous behaviour.

/// Each scenario which a *previous* settings object maps to. Which migration takes place depends
/// the resulting scenario.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scenario {
    OneA,   // 1A
    OneB,   // 1B
    Two,    // 2
    ThreeA, // 3A
    ThreeB, // 3B
    FourA,  // 4A
    FourB,  // 4B
    FiveA,  // 5A
    FiveB,  // 5B
    SixA,   // 6A
    SixB,   // 6B
    SevenA, // 7A
    SevenB, // 7B
}
