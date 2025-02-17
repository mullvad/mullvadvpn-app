//
//  APITransportProvider.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public protocol APITransportProviderProtocol {
    func makeTransport() -> APITransportProtocol?
}

public final class APITransportProvider: APITransportProviderProtocol, Sendable {
    let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func makeTransport() -> APITransportProtocol? {
        APITransport(requestFactory: requestFactory)
    }
}

extension REST {
    public struct AnyAPITransportProvider: APITransportProviderProtocol {
        private let block: () -> APITransportProtocol?

        public init(_ block: @escaping @Sendable () -> APITransportProtocol?) {
            self.block = block
        }

        public func makeTransport() -> APITransportProtocol? {
            return block()
        }
    }
}
