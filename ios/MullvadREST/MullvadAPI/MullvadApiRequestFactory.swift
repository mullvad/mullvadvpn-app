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
        case let .sendProblemReport(retryStrategy, problemReportRequest):
            return MullvadApiCancellable(
                handle: apiContext.sendProblemReport(
                    retryStrategy: retryStrategy.toRustStrategy(),
                    request: problemReportRequest
                ))

        // Device Proxy
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
