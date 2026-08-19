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

public class MullvadApiContext: @unchecked Sendable, ApiContextCallbackContext, ApiContextCallback {
    public func accessMethodChange(context: any ApiContextCallbackContext, uuid: Data) {
        guard let self = context as? MullvadApiContext else { return }

        let parsedUUID = uuid.withUnsafeBytes { (ptr: UnsafeRawBufferPointer) in
            NSUUID(uuidBytes: ptr.baseAddress) as UUID
        }
        self.accessMethodChangeListeners.forEach { $0.accessMethodChangedTo(parsedUUID) }
    }

    enum Error: Swift.Error {
        case failedToConstructApiClient
    }

    public private(set) var context: ApiContext!
    private let shadowsocksBridgeProvider: ShadowsocksBridgeProvider!
    public let accessMethodChangeListeners: [MullvadAccessMethodChangeListening]

    public init(
        host: String,
        address: String,
        domain: String,
        disableTls: Bool = false,
        shadowsocksProvider: ShadowsocksBridgeProvider,
        accessMethodWrapper: SwiftAccessMethodSettingsContext,
        accessMethodChangeListeners: [MullvadAccessMethodChangeListening]
    ) throws {
        self.shadowsocksBridgeProvider = shadowsocksProvider

        self.accessMethodChangeListeners = accessMethodChangeListeners

        context =
            switch disableTls {
            case true:
                ApiContext.newTlsDisabled(
                    host: host,
                    address: address,
                    domain: domain,
                    bridgeProvider: shadowsocksProvider,
                    settingsProvider: accessMethodWrapper,
                    accessMethodChangeCallback: self,
                    accessMethodChangeContext: self)
            case false:
                ApiContext(
                    host: host,
                    address: address,
                    domain: domain,
                    bridgeProvider: shadowsocksProvider,
                    settingsProvider: accessMethodWrapper,
                    accessMethodChangeCallback: self,
                    accessMethodChangeContext: self)
            }
    }
}
