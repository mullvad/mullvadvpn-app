// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import SwiftUI

@Observable final class HeaderBarViewModel {
    var rootConfiguration: RootConfiguration
    var headerBarPresentation: HeaderBarPresentation
    var breadcrumb: Breadcrumb?
    var showDeviceInfo: Bool = true

    var onAccountTap: (() -> Void)?
    var onSettingsTap: (() -> Void)?

    init(
        rootConfiguration: RootConfiguration,
        headerBarPresentation: HeaderBarPresentation,
        breadcrumb: Breadcrumb?
    ) {
        self.rootConfiguration = rootConfiguration
        self.headerBarPresentation = headerBarPresentation
        self.breadcrumb = breadcrumb
    }

    var backgroundColor: Color {
        headerBarPresentation.style.backgroundColor()
    }

    var showAccountButton: Bool {
        rootConfiguration.showsAccountButton
    }

    var deviceName: String {
        if let deviceName = rootConfiguration.deviceName {
            String(format: NSLocalizedString("Device name: %@", comment: ""), deviceName)
        } else {
            ""
        }
    }

    var timeLeft: String {
        if let expiry = rootConfiguration.expiry {
            String(
                format: NSLocalizedString("Time left: %@", comment: ""),
                CustomDateComponentsFormatting.localizedString(
                    from: Date(),
                    to: expiry,
                    unitsStyle: .full
                ) ?? ""
            )
        } else {
            ""
        }
    }
}
