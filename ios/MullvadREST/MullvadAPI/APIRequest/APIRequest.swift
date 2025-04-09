//
//  APIRequest.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes

public enum APIRequest: Codable, Sendable {
    case getAddressList(_ retryStrategy: REST.RetryStrategy)
    case getRelayList(_ retryStrategy: REST.RetryStrategy, etag: String?)
    case sendProblemReport(_ retryStrategy: REST.RetryStrategy, problemReportRequest: ProblemReportRequest)

    case createAccount(_ retryStrategy: REST.RetryStrategy)
    case getAccount(_ retryStrategy: REST.RetryStrategy, accountNumber: String)
    case deleteAccount(_ retryStrategy: REST.RetryStrategy, accountNumber: String)

    var name: String {
        switch self {
        case .getAddressList:
            "get-address-list"
        case .getRelayList:
            "get-relay-list"
        case .sendProblemReport:
            "send-problem-report"
        case .createAccount:
            "create-account"
        case .getAccount:
            "get-account"
        case .deleteAccount:
            "delete-account"
        }
    }

    var retryStrategy: REST.RetryStrategy {
        switch self {
        case
            let .getAddressList(strategy),
            let .getRelayList(strategy, _),
            let .sendProblemReport(strategy, _),
            let .createAccount(strategy),
            let .getAccount(strategy, _),
            let .deleteAccount(strategy, _):
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
    public let etag: String?

    public init(data: Data?, error: APIError?, etag: String? = nil) {
        self.data = data
        self.error = error
        self.etag = etag
    }
}
