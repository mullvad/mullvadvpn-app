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

/// Content configuration presenting a label and switch control.
struct ListCellContentConfiguration: UIContentConfiguration, Equatable {
    struct TextProperties: Equatable {
        var font = UIFont.mullvadSmall
        var color = UIColor.Cell.titleTextColor
    }

    struct SecondaryTextProperties: Equatable {
        var font = UIFont.mullvadSmall
        var color = UIColor.Cell.detailTextColor.withAlphaComponent(0.6)
    }

    struct TertiaryTextProperties: Equatable {
        var font = UIFont.mullvadTiny
        var color = UIColor.Cell.titleTextColor.withAlphaComponent(0.6)
    }

    /// Primary text label.
    var text: String?
    var textProperties = TextProperties()

    /// Secondary (trailing) text label.
    var secondaryText: String?
    var secondaryTextProperties = SecondaryTextProperties()

    /// Tertiary (below primary) text label.
    var tertiaryText: String?
    var tertiaryTextProperties = TertiaryTextProperties()

    /// Content view layout margins.
    var directionalLayoutMargins = UIMetrics.SettingsCell.defaultLayoutMargins

    /// Enabled state.
    var isEnabled = true

    /// Selected state.
    var isSelected = false

    /// Breadcrumb.
    var showBreadcrumb = false

    func makeContentView() -> UIView & UIContentView {
        return ListCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
