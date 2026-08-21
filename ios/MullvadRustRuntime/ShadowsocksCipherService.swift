// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import MullvadTypes

public struct ShadowsocksCipherService {
    public init() {}

    public func getCiphers() -> [String] {
        guard let pointer = get_shadowsocks_chipers() else {
            Logger(label: "ShadowsocksCipherService").error("Failed to get Shadowsocks ciphers")
            return []
        }

        let cipherString = String(cString: pointer)
        mullvad_api_cstring_drop(pointer)

        return cipherString.components(separatedBy: ",").sorted()
    }
}
