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
    public let apiContext: MullvadApiContext
    private let encoder: JSONEncoder

    public init(apiContext: MullvadApiContext, encoder: JSONEncoder) {
        self.apiContext = apiContext
        self.encoder = encoder
    }

    public func makeRequest(_ request: APIRequest) throws -> MullvadApiCancellable {
        switch request {
        case let .getAddressList(retryStrategy):
            return MullvadApiCancellable(
                handle: mullvadIosGetAddresses(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy()
                ))

        case let .getRelayList(retryStrategy, etag: etag):
            return MullvadApiCancellable(
                handle: mullvadIosGetRelays(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    etag: etag
                ))
        case let .sendProblemReport(retryStrategy, problemReportRequest):
            return MullvadApiCancellable(
                handle: mullvadIosSendProblemReport(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    request: problemReportRequest
                ))
        case let .getAccount(retryStrategy, accountNumber: accountNumber):
            return MullvadApiCancellable(
                handle: mullvadIosGetAccount(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))
        case let .createAccount(retryStrategy):
            return MullvadApiCancellable(
                handle: mullvadIosCreateAccount(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy()
                ))
        case let .deleteAccount(retryStrategy, accountNumber: accountNumber):
            return MullvadApiCancellable(
                handle: mullvadIosDeleteAccount(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))

        // Device Proxy
        case let .getDevice(retryStrategy, accountNumber: accountNumber, identifier):
            return MullvadApiCancellable(
                handle: mullvadIosGetDevice(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    identifier: identifier
                ))

        case let .getDevices(retryStrategy, accountNumber):
            return MullvadApiCancellable(
                handle: mullvadIosGetDevices(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))

        case let .deleteDevice(retryStrategy, accountNumber, identifier):
            return MullvadApiCancellable(
                handle: mullvadIosDeleteDevice(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    identifier: identifier
                ))
        case let .rotateDeviceKey(retryStrategy, accountNumber, identifier, publicKey):
            return MullvadApiCancellable(
                handle: mullvadIosRotateDeviceKey(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    identifier: identifier,
                    publicKey: publicKey.rawValue
                ))
        case let .createDevice(retryStrategy, accountNumber, request):
            return MullvadApiCancellable(
                handle: mullvadIosCreateDevice(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber,
                    publicKey: request.publicKey.rawValue
                ))
        case let .checkApiAvailability(retryStrategy, accessMethod):
            return MullvadApiCancellable(
                handle: mullvadIosApiAddrsAvailable(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accessMethodSetting: convertAccessMethod(accessMethod: accessMethod)!
                ))
        case let .initStorekitPayment(
            retryStrategy: retryStrategy,
            accountNumber: accountNumber
        ):
            return MullvadApiCancellable(
                handle: mullvadIosInitStorekitPayment(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    accountNumber: accountNumber
                ))
        case let .checkStorekitPayment(
            retryStrategy: retryStrategy,
            transaction: transaction
        ):
            return MullvadApiCancellable(
                handle: mullvadIosCheckStorekitPayment(
                    apiContext: apiContext.context,
                    retryStrategy: retryStrategy.toRustStrategy(),
                    body: try encoder.encode(transaction)
                ))
        }
    }
}
