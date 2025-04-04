//
//  MullvadAccountProxy.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-03-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadTypes
import Operations
import WireGuardKitTypes

public protocol RESTAccountHandling: Sendable {
    func createAccount(
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<REST.NewAccountData>
    ) -> Cancellable

    func getAccountData(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping @Sendable ProxyCompletionHandler<Account>
    ) -> Cancellable

    func deleteAccount(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable
}

extension REST {
    public struct NewAccountData: Decodable, Sendable {
        public let id: String
        public let expiry: Date
        public let maxPorts: Int
        public let canAddPorts: Bool
        public let maxDevices: Int
        public let canAddDevices: Bool
        public let number: String
    }

    public final class MullvadAccountProxy: RESTAccountHandling, @unchecked Sendable {
        let transportProvider: APITransportProviderProtocol
        let dispatchQueue: DispatchQueue
        let operationQueue = AsyncOperationQueue()
        let responseDecoder: JSONDecoder

        public init(
            transportProvider: APITransportProviderProtocol,
            dispatchQueue: DispatchQueue,
            responseDecoder: JSONDecoder
        ) {
            self.transportProvider = transportProvider
            self.dispatchQueue = dispatchQueue
            self.responseDecoder = responseDecoder
        }

        public func createAccount(
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<NewAccountData>
        ) -> Cancellable {
            let responseHandler = rustResponseHandler(
                decoding: NewAccountData.self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .createAccount(retryStrategy),
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        public func getAccountData(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Account>
        ) -> Cancellable {
            let responseHandler = rustResponseHandler(
                decoding: Account.self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .getAccount(retryStrategy, accountNumber: accountNumber),
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        public func deleteAccount(
            accountNumber: String,
            retryStrategy: RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Void>
        ) -> Cancellable {
            let request = APIRequest.deleteAccount(retryStrategy, accountNumber: accountNumber)

            let networkOperation = MullvadApiNetworkOperation(
                name: request.name,
                dispatchQueue: dispatchQueue,
                request: request,
                transportProvider: transportProvider,
                responseDecoder: responseDecoder,
                responseHandler: rustEmptyResponseHandler(),
                completionHandler: completion
            )

            operationQueue.addOperation(networkOperation)

            return networkOperation
        }

        private func createNetworkOperation<Success: Decodable>(
            request: APIRequest,
            responseHandler: RustResponseHandler<Success>,
            completionHandler: @escaping @Sendable ProxyCompletionHandler<Success>
        ) -> MullvadApiNetworkOperation<Success> {
            let networkOperation = MullvadApiNetworkOperation(
                name: request.name,
                dispatchQueue: dispatchQueue,
                request: request,
                transportProvider: transportProvider,
                responseDecoder: responseDecoder,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )

            operationQueue.addOperation(networkOperation)

            return networkOperation
        }
    }
}

extension REST.NewAccountData {
    public static func mockValue() -> REST.NewAccountData {
        return REST.NewAccountData(
            id: UUID().uuidString,
            expiry: Date().addingTimeInterval(3600),
            maxPorts: 2,
            canAddPorts: false,
            maxDevices: 5,
            canAddDevices: false,
            number: "1234567890123456"
        )
    }
}
