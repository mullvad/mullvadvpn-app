//! Since the old multihop settings do not map cleanly onto the new multihop settings and semantics, this migration performs breaking changes to user settings. But it tries really hard not to!
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
//! # Note
//! - The scenario naming scheme is inherited from the UI/UX team at Mullvad.
//! - Filters are considered because they preivously affected DAITA through 'Magic Multihop" / 'automatic multihop'.
//!   This was undefined (or atleast undocumented) behaviour, but it was decided when this migration was architected to respect the previous behaviour.
//! - `Multihop` was previously a boolean value, but it will be migrated to a tri-nary [Multihop] setting.
//! - [WhenNeeded] generalizes the previous 'automatic multihop' setting to other settings than
//!   DAITA.
//! - [Always] and [Never] maps more trivially from the old multihop boolean value when DAITA was
//!   disabled.
//!
//! [Multihop]: crate::migrations::multihop::settings::v18::__Multihop
//! [WhenNeeded]: crate::migrations::multihop::settings::v18::__Multihop::WhenNeeded
//! [Always]: crate::migrations::multihop::settings::v18::__Multihop::Always
//! [Never]: crate::migrations::multihop::settings::v18::__Multihop::Never

use crate::migrations::multihop::settings::__LocationConstraint;

/// Each scenario which a [*previous* settings](crate::migrations::multihop::settings::v17) object maps to.
/// Exactly how the migration is executed depends on this scenario.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Scenario {
    OneA,   // 1A
    OneB,   // 1B
    Two,    // 2
    ThreeA, // 3A
    // 3B
    ThreeB {
        #[serde(skip)]
        last_known_working_location: __LocationConstraint,
    },
    FourA,  // 4A
    FourB,  // 4B
    FiveA,  // 5A
    FiveB,  // 5B
    SixA,   // 6A
    SixB,   // 6B
    SevenA, // 7A
    SevenB, // 7B
}
