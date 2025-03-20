//
//  APIRequest.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public enum APIRequest: Codable, Sendable {
    case getAddressList(_ retryStrategy: REST.RetryStrategy)
    case sendProblemReport(_ retryStrategy: REST.RetryStrategy, problemReportRequest: REST.ProblemReportRequest)

    var retryStrategy: REST.RetryStrategy {
        return switch self {
        case let .getAddressList(strategy):
            strategy
        case let .sendProblemReport(strategy, _):
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
    public let data: Data?
    public let error: APIError?

    public init(data: Data?, error: APIError?) {
        self.data = data
        self.error = error
    }
}
