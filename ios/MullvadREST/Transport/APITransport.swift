//
//  APITransport.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2025-02-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadTypes

public protocol APITransportProtocol {
    var name: String { get }

    func sendRequest(_ request: APIRequest, completion: @escaping @Sendable (ProxyAPIResponse) -> Void)
        -> Cancellable
}

public final class APITransport: APITransportProtocol {
    public var name: String {
        "app-transport"
    }

    public let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func sendRequest(
        _ request: APIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) -> Cancellable {
        let apiRequest = requestFactory.makeRequest(request)

        return apiRequest { response in
            let error: APIError? = if response.statusCode != 200 {
                APIError(
                    statusCode: Int(response.statusCode),
                    errorDescription: response.errorDescription ?? "",
                    serverResponseCode: response.serverResponseCode
                )
            } else { nil }

            completion(ProxyAPIResponse(
                data: response.body,
                error: error,
                etag: response.etag
            ))
        }
    }
}
