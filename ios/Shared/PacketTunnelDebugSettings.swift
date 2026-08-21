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

#if DEBUG
    /// Debug settings for switching between packet tunnel implementations.
    /// Stored in the shared App Group UserDefaults so both the main app and the
    /// packet tunnel extension can access them.
    enum PacketTunnelDebugSettings {
        private static let useGotaTunKey = "PacketTunnelDebugSettings.useGotaTun"

        private static var sharedDefaults: UserDefaults? {
            UserDefaults(suiteName: ApplicationConfiguration.securityGroupIdentifier)
        }

        /// Whether the GotaTun adapter should be used instead of WireGuard.
        /// Defaults to `false` if the shared container is unavailable.
        static var useGotaTun: Bool {
            get {
                sharedDefaults?.bool(forKey: useGotaTunKey) ?? false
            }
            set {
                sharedDefaults?.set(newValue, forKey: useGotaTunKey)
            }
        }
    }
#endif
