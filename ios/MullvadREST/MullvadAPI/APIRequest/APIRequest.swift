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

public enum APIRequest: Codable, Sendable {
    // Api Proxy
    case sendProblemReport(_ retryStrategy: REST.RetryStrategy, problemReportRequest: ProblemReportRequest)

    // Account Proxy
    case initStorekitPayment(retryStrategy: REST.RetryStrategy, accountNumber: String)
    case checkStorekitPayment(retryStrategy: REST.RetryStrategy, transaction: StoreKitTransaction)

    // Device Proxy
    case rotateDeviceKey(
        _ retryStrategy: REST.RetryStrategy,
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey
    )

    var name: String {
        switch self {
        case .sendProblemReport:
            "send-problem-report"
        case .rotateDeviceKey:
            "rotate-device-key"
        case .initStorekitPayment:
            "init-storekit-payment"
        case .checkStorekitPayment:
            "check-storekit-payment"
        }
    }

    var retryStrategy: REST.RetryStrategy {
        switch self {
        case let .sendProblemReport(strategy, _),
            let .rotateDeviceKey(strategy, _, _, _),
            let .initStorekitPayment(strategy, _),
            let .checkStorekitPayment(strategy, _):
            strategy
        }
    }
}

public struct ProxyAPIRequest: Codable, Sendable {
    public let id: UUID
    public let request: APIRequest

    public init(id: UUID, request: APIRequest) {
        self.id = id
        self.request = request
    }
}

public struct ProxyAPIResponse: Codable, Sendable {
    public let digest: String?
    public let timestamp: Int64?
    public let data: Data?
    public let error: APIError?

    public init(data: Data?, error: APIError?, digest: String? = nil, timestamp: Int64? = nil) {
        self.digest = digest
        self.timestamp = timestamp
        self.data = data
        self.error = error
    }
}
