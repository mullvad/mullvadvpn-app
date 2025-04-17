//
//  APIRequest.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes
@preconcurrency import WireGuardKitTypes

public enum APIRequest: Codable, Sendable {
    // Api Proxy
    case getAddressList(_ retryStrategy: REST.RetryStrategy)
    case getRelayList(_ retryStrategy: REST.RetryStrategy, etag: String?)
    case sendProblemReport(_ retryStrategy: REST.RetryStrategy, problemReportRequest: ProblemReportRequest)

    // Account Proxy
    case createAccount(_ retryStrategy: REST.RetryStrategy)
    case getAccount(_ retryStrategy: REST.RetryStrategy, accountNumber: String)
    case deleteAccount(_ retryStrategy: REST.RetryStrategy, accountNumber: String)

    // Device Proxy
    case getDevice(_ retryStrategy: REST.RetryStrategy, accountNumber: String, identifier: String)
    case getDevices(_ retryStrategy: REST.RetryStrategy, accountNumber: String)
    case createDevice(_ retryStrategy: REST.RetryStrategy, accountNumber: String, request: CreateDeviceRequest)
    case deleteDevice(_ retryStrategy: REST.RetryStrategy, accountNumber: String, identifier: String)
    case rotateDeviceKey(
        _ retryStrategy: REST.RetryStrategy,
        accountNumber: String,
        identifier: String,
        publicKey: PublicKey
    )

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
        case .getDevice:
            "get-device"
        case .getDevices:
            "get-devices"
        case .deleteDevice:
            "delete-device"
        case .rotateDeviceKey:
            "rotate-device-key"
        case .createDevice:
            "create-device"
        }
    }

    var retryStrategy: REST.RetryStrategy {
        switch self {
        case let .getAddressList(strategy),
             let .getRelayList(strategy, _),
             let .sendProblemReport(strategy, _),
             let .createAccount(strategy),
             let .getAccount(strategy, _),
             let .deleteAccount(strategy, _),
             let .createDevice(strategy, _, _),
             let .getDevice(strategy, _, _),
             let .getDevices(strategy, _),
             let .deleteDevice(strategy, _, _),
             let .rotateDeviceKey(strategy, _, _, _):
            return strategy
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
