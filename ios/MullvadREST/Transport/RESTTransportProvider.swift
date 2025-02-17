//
//  RESTTransportProvider.swift
//  MullvadREST
//
//  Created by pronebird on 24/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

public protocol RESTTransportProvider {
    func makeTransport() -> RESTTransport?
}

extension REST {
    public struct AnyTransportProvider: RESTTransportProvider {
        private let block: () -> RESTTransport?

        public init(_ block: @escaping @Sendable () -> RESTTransport?) {
            self.block = block
        }

        public func makeTransport() -> RESTTransport? {
            return block()
        }
    }
}

public protocol APITransportProviderProtocol {
    func makeTransport() -> APITransportProtocol?
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

public final class APITransportProvider: APITransportProviderProtocol, Sendable {
    let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func makeTransport() -> APITransportProtocol? {
        APITransport(requestFactory: requestFactory)
    }
}
