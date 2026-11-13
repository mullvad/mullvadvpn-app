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

extension PersistentAccessMethod {
    static func makePersistentAccessMethod() -> PersistentAccessMethod {
        PersistentAccessMethod(
            id: UUID(),
            name: UUID().uuidString,
            isEnabled: true,
            proxyConfiguration: [
                PersistentProxyConfiguration.direct,
                PersistentProxyConfiguration.bridges,
            ].randomElement()!
        )
    }
}

extension ListAccessMethodItem {
    static func makeAccessMethodItems() -> [ListAccessMethodItem] {
        (0..<10).map { _ in
            let item = PersistentAccessMethod.makePersistentAccessMethod()
            return ListAccessMethodItem(
                id: item.id, name: item.name, detail: nil, isEnabled: item.isEnabled, status: .on)
        }
    }
}
