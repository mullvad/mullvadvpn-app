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

/// Content configuration for presenting the access method testing progress.
struct MethodTestingStatusCellContentConfiguration: UIContentConfiguration, Equatable {
    /// The status of access method testing.
    enum Status: Equatable {
        /// API Is reachable.
        case reachable

        /// API is unreachable.
        case unreachable

        /// API testing is in progress.
        case testing
    }

    /// The status of testing.
    var status: Status = .reachable

    /// Detail text displayed below the status when set.
    var detailText: String?

    /// Layout margins.
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.defaultLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return MethodTestingStatusCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}

extension MethodTestingStatusCellContentConfiguration.Status {
    /// The text label descirbing the status of testing and suitable for user presentation.
    var text: String {
        switch self {
        case .unreachable:
            NSLocalizedString("API unreachable", comment: "")
        case .reachable:
            NSLocalizedString("API reachable", comment: "")
        case .testing:
            NSLocalizedString("Testing...", comment: "")
        }
    }

    /// The color of a circular status indicator view.
    var statusColor: UIColor? {
        switch self {
        case .unreachable:
            .dangerColor
        case .reachable:
            .successColor
        case .testing:
            nil
        }
    }
}
