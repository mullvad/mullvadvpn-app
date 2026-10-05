// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

/// Types handling custom disclosure accessory in table view cells.
protocol CustomCellDisclosureHandling: UITableViewCell {
    /// Custom disclosure type.
    ///
    /// Cannot be used together with `accessoryType` property. Automatically resets `accessoryType` upon assignment.
    var disclosureType: SettingsDisclosureType { get set }
}
