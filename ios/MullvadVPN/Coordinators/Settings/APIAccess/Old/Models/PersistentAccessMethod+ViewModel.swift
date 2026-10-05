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
import MullvadSettings
import MullvadTypes

extension PersistentAccessMethod {
    /// Convert persistent model into view model.
    /// - Returns: an instance of ``AccessMethodViewModel``.
    func toViewModel() -> AccessMethodViewModel {
        AccessMethodViewModel(
            id: id,
            name: name,
            method: kind,
            isEnabled: isEnabled,
            socks: proxyConfiguration.socksViewModel,
            shadowsocks: proxyConfiguration.shadowsocksViewModel
        )
    }
}
