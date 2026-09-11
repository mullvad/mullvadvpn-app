// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadRustRuntime
import MullvadTypes

public struct MullvadApiRequestFactory: Sendable {
    public let apiContext: ApiContext
    private let encoder: JSONEncoder

    public init(apiContext: ApiContext, encoder: JSONEncoder) {
        self.apiContext = apiContext
        self.encoder = encoder
    }

    public func makeRequest(_ request: APIRequest) throws -> MullvadApiCancellable {
        switch request {
        case let .getAddressList(retryStrategy):
            return MullvadApiCancellable(
                handle: apiContext.getAddresses(
                    retryStrategy: retryStrategy.toRustStrategy()
                ))

        case let .getRelayList(retryStrategy, etag: etag):
            return MullvadApiCancellable(
                handle: apiContext.getRelays(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    etag: etag
                ))
        case let .sendProblemReport(retryStrategy, problemReportRequest):
            return MullvadApiCancellable(
                handle: apiContext.sendProblemReport(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    request: problemReportRequest
                ))
        case let .getAccount(retryStrategy, accountNumber: accountNumber):
            return MullvadApiCancellable(
                handle: apiContext.getAccount(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))
        case let .createAccount(retryStrategy):
            return MullvadApiCancellable(
                handle: apiContext.createAccount(
                    retryStrategy: retryStrategy.toRustStrategy()
                ))
        case let .deleteAccount(retryStrategy, accountNumber: accountNumber):
            return MullvadApiCancellable(
                handle: apiContext.deleteAccount(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))

        // Device Proxy
        case let .getDevice(retryStrategy, accountNumber: accountNumber, identifier):
            return MullvadApiCancellable(
                handle: apiContext.getDevice(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    identifier: identifier
                ))

        case let .getDevices(retryStrategy, accountNumber):
            return MullvadApiCancellable(
                handle: apiContext.getDevices(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))

        case let .deleteDevice(retryStrategy, accountNumber, identifier):
            return MullvadApiCancellable(
                handle: apiContext.deleteDevice(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    identifier: identifier
                ))
        case let .rotateDeviceKey(retryStrategy, accountNumber, identifier, publicKey):
            return MullvadApiCancellable(
                handle: apiContext.rotateDeviceKey(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    identifier: identifier,
                    publicKey: publicKey.rawValue
                ))
        case let .createDevice(retryStrategy, accountNumber, request):
            return MullvadApiCancellable(
                handle: apiContext.createDevice(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    publicKey: request.publicKey.rawValue
                ))
        case let .checkApiAvailability(retryStrategy, accessMethod):
            return MullvadApiCancellable(
                handle: apiContext.apiAddrsAvailable(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accessMethodSetting: convertAccessMethod(accessMethod: accessMethod)!
                ))
        case let .initStorekitPayment(
            retryStrategy: retryStrategy,
            accountNumber: accountNumber
        ):
            return MullvadApiCancellable(
                handle: apiContext.initStorekitPayment(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))
        case let .checkStorekitPayment(
            retryStrategy: retryStrategy,
            transaction: transaction
        ):
            return MullvadApiCancellable(
                handle: apiContext.checkStorekitPayment(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    body: try encoder.encode(transaction)
                ))
        }
    }
}
