//
//  RESTProxy.swift
//  MullvadREST
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes
import Operations

public typealias ProxyCompletionHandler<Success: Sendable> = @Sendable (Result<Success, Swift.Error>) -> Void

extension REST {
    public class Proxy<ConfigurationType: ProxyConfiguration>: @unchecked Sendable {
        /// Synchronization queue used by network operations.
        let dispatchQueue: DispatchQueue

        /// Operation queue used for running network operations.
        let operationQueue = AsyncOperationQueue()

        /// Proxy configuration.
        let configuration: ConfigurationType

        /// URL request factory.
        let requestFactory: REST.RequestFactory

        let mullvadApiRequestFactory: MullvadApiRequestFactory

        /// URL response decoder.
        let responseDecoder: JSONDecoder

        init(
            name: String,
            configuration: ConfigurationType,
            requestFactory: REST.RequestFactory,
            responseDecoder: JSONDecoder,
            requestEncoder: JSONEncoder
        ) {
            dispatchQueue = DispatchQueue(label: "REST.\(name).dispatchQueue")
            operationQueue.name = "REST.\(name).operationQueue"

            self.configuration = configuration
            self.requestFactory = requestFactory
            self.mullvadApiRequestFactory = MullvadApiRequestFactory(
                apiContext: configuration.apiContext,
                encoder: requestEncoder
            )
            self.responseDecoder = responseDecoder
        }

        func makeRequestExecutor<Success: Sendable>(
            name: String,
            requestHandler: RESTRequestHandler,
            responseHandler: some RESTResponseHandler<Success>
        ) -> any RESTRequestExecutor<Success> {
            let operationFactory = NetworkOperationFactory(
                dispatchQueue: dispatchQueue,
                configuration: configuration,
                name: name,
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return RequestExecutor(operationFactory: operationFactory, operationQueue: operationQueue)
        }
    }

    /// Factory object producing instances of `NetworkOperation`.
    private struct NetworkOperationFactory<Success: Sendable, ConfigurationType: ProxyConfiguration> {
        let dispatchQueue: DispatchQueue
        let configuration: ConfigurationType

        let name: String
        let requestHandler: RESTRequestHandler
        let responseHandler: any RESTResponseHandler<Success>

        /// Creates new network operation but does not schedule it for execution.
        func makeOperation(
            retryStrategy: REST.RetryStrategy,
            completionHandler: ProxyCompletionHandler<Success>? = nil
        ) -> NetworkOperation<Success> {
            return NetworkOperation(
                name: getTaskIdentifier(name: name),
                dispatchQueue: dispatchQueue,
                configuration: configuration,
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )
        }
    }

    /// Network request executor that supports block-based and async execution flows.
    private struct RequestExecutor<Success: Sendable, ConfigurationType: ProxyConfiguration>: RESTRequestExecutor {
        let operationFactory: NetworkOperationFactory<Success, ConfigurationType>
        let operationQueue: AsyncOperationQueue

        func execute(
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping ProxyCompletionHandler<Success>
        ) -> Cancellable {
            let operation = operationFactory.makeOperation(
                retryStrategy: retryStrategy,
                completionHandler: completionHandler
            )

            operationQueue.addOperation(operation)

            return operation
        }

        func execute(retryStrategy: REST.RetryStrategy) async throws -> Success {
            let operation = operationFactory.makeOperation(retryStrategy: retryStrategy)

            return try await withTaskCancellationHandler {
                return try await withCheckedThrowingContinuation { continuation in
                    operation.completionHandler = { result in
                        continuation.resume(with: result)
                    }
                    operationQueue.addOperation(operation)
                }
            } onCancel: {
                operation.cancel()
            }
        }

        func execute(completionHandler: @escaping @Sendable ProxyCompletionHandler<Success>) -> Cancellable {
            return execute(retryStrategy: .noRetry, completionHandler: completionHandler)
        }

        func execute() async throws -> Success {
            return try await execute(retryStrategy: .noRetry)
        }
    }

    public class ProxyConfiguration: @unchecked Sendable {
        public let transportProvider: RESTTransportProvider
        public let addressCacheStore: AddressCache
        public let apiContext: MullvadApiContext

        public init(
            transportProvider: RESTTransportProvider,
            addressCacheStore: AddressCache,
            apiContext: MullvadApiContext
        ) {
            self.transportProvider = transportProvider
            self.addressCacheStore = addressCacheStore
            self.apiContext = apiContext
        }
    }

    public class AuthProxyConfiguration: ProxyConfiguration, @unchecked Sendable {
        public let accessTokenManager: RESTAccessTokenManagement

        public init(
            proxyConfiguration: ProxyConfiguration,
            accessTokenManager: RESTAccessTokenManagement,
            apiContext: MullvadApiContext
        ) {
            self.accessTokenManager = accessTokenManager

            super.init(
                transportProvider: proxyConfiguration.transportProvider,
                addressCacheStore: proxyConfiguration.addressCacheStore,
                apiContext: apiContext
            )
        }
    }
}
