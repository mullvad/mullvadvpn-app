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

// Wrapper around a `MullvadAccessMethodChangeListening` to remove the need for every
// listener to parse the uuid.
private final class CallbackShim: AccessMethodChangeCallback {
    func accessMethodChangedTo(uuid: Data) {
        let parsedUUID = uuid.withUnsafeBytes { (ptr: UnsafeRawBufferPointer) in
            NSUUID(uuidBytes: ptr.baseAddress) as UUID
        }
        inner.accessMethodChangedTo(parsedUUID)
    }

    let inner: any MullvadAccessMethodChangeListening
    init(_ inner: any MullvadAccessMethodChangeListening) {
        self.inner = inner
    }
}

extension ApiContext {
    public convenience init(
        host: String,
        address: String,
        domain: String,
        disableTls: Bool = false,
        shadowsocksProvider: ShadowsocksBridgeProvider,
        accessMethodWrapper: SwiftAccessMethodSettingsContext,
        accessMethodChangeListeners: [any MullvadAccessMethodChangeListening]
    ) {
        self.init(
            host: host,
            address: address,
            domain: domain,
            disableTls: disableTls,
            bridgeProvider: shadowsocksProvider,
            settingsProvider: accessMethodWrapper,
            accessMethodChangeListeners: accessMethodChangeListeners.map(CallbackShim.init))
    }
}
