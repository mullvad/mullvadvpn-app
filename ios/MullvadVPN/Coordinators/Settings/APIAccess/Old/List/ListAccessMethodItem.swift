// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

/// A concrete implementation of an API access list item.
struct ListAccessMethodItem: Hashable, Identifiable, Equatable {
    /// The unique ID.
    let id: UUID

    /// The localized name.
    let name: String

    /// The detailed information displayed alongside.
    let detail: String?

    /// Whether method is enabled or not.
    let isEnabled: Bool
}
