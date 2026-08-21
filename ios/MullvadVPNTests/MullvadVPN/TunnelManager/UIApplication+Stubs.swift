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
import UIKit

@testable import MullvadTypes

final class UIApplicationStub: BackgroundTaskProviding {
    var backgroundTimeRemaining: TimeInterval { .infinity }

    func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier) {}

    #if compiler(>=6)
        func beginBackgroundTask(
            withName taskName: String?,
            expirationHandler handler: (@MainActor @Sendable () -> Void)?
        )
            -> UIBackgroundTaskIdentifier
        {
            .invalid
        }
    #else
        func beginBackgroundTask(
            withName taskName: String?,
            expirationHandler handler: (() -> Void)?
        ) -> UIBackgroundTaskIdentifier {
            .invalid
        }
    #endif
}
