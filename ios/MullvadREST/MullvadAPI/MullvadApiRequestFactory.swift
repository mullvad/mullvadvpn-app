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
                handle: mullvad_ios_get_addresses(
                    apiContext.context,
                    retryStrategy.toRustStrategy()
                ))

        case let .getRelayList(retryStrategy, etag: etag):
            return MullvadApiCancellable(
                handle: mullvad_ios_get_relays(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    etag
                ))
        case let .sendProblemReport(retryStrategy, problemReportRequest):
            let rustRequest = RustProblemReportRequest(from: problemReportRequest)
            return MullvadApiCancellable(
                handle: mullvad_ios_send_problem_report(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    rustRequest.toRust()
                ))
        case let .getAccount(retryStrategy, accountNumber: accountNumber):
            return MullvadApiCancellable(
                handle: mullvad_ios_get_account(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
        case let .createAccount(retryStrategy):
            return MullvadApiCancellable(
                handle: mullvad_ios_create_account(
                    apiContext.context,
                    retryStrategy.toRustStrategy()
                ))
        case let .deleteAccount(retryStrategy, accountNumber: accountNumber):
            return MullvadApiCancellable(
                handle: mullvad_ios_delete_account(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))

        // Device Proxy
        case let .getDevice(retryStrategy, accountNumber: accountNumber, identifier):
            return MullvadApiCancellable(
                handle: mullvad_ios_get_device(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    identifier
                ))

        case let .getDevices(retryStrategy, accountNumber):
            return MullvadApiCancellable(
                handle: mullvad_ios_get_devices(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))

        case let .deleteDevice(retryStrategy, accountNumber, identifier):
            return MullvadApiCancellable(
                handle: mullvad_ios_delete_device(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    identifier
                ))
        case let .rotateDeviceKey(retryStrategy, accountNumber, identifier, publicKey):
            return MullvadApiCancellable(
                handle: mullvad_ios_rotate_device_key(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    identifier,
                    publicKey.rawValue.map { $0 }
                ))
        case let .createDevice(retryStrategy, accountNumber, request):
            return MullvadApiCancellable(
                handle: mullvad_ios_create_device(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    request.publicKey.rawValue.map { $0 }
                ))
        case let .checkApiAvailability(retryStrategy, accessMethod):
            return MullvadApiCancellable(
                handle: mullvad_ios_api_addrs_available(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    convertAccessMethod(accessMethod: accessMethod)
                ))
        case let .initStorekitPayment(
            retryStrategy: retryStrategy,
            accountNumber: accountNumber
        ):
            return MullvadApiCancellable(
                handle: mullvad_ios_init_storekit_payment(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
        case let .checkStorekitPayment(
            retryStrategy: retryStrategy,
            transaction: transaction
        ):
            let body = try encoder.encode(transaction)
            return MullvadApiCancellable(
                handle: mullvad_ios_check_storekit_payment(
                    apiContext.context,
                    retryStrategy.toRustStrategy(),
                    body.map { $0 },
                    UInt(body.count)
                ))
        }
    }
}
