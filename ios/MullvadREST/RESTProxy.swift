//
//  RESTProxy.swift
//  MullvadREST
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Operations

extension REST {
    public class Proxy<ConfigurationType: ProxyConfiguration> {
        public typealias CompletionHandler<Success> = (Result<Success, Swift.Error>) -> Void

        /// Synchronization queue used by network operations.
        let dispatchQueue: DispatchQueue

        /// Operation queue used for running network operations.
        let operationQueue = AsyncOperationQueue()

        /// Proxy configuration.
        let configuration: ConfigurationType

        /// URL request factory.
        let requestFactory: REST.RequestFactory

        /// URL response decoder.
        let responseDecoder: JSONDecoder

        init(
            name: String,
            configuration: ConfigurationType,
            requestFactory: REST.RequestFactory,
            responseDecoder: JSONDecoder
        ) {
            dispatchQueue = DispatchQueue(label: "REST.\(name).dispatchQueue")
            operationQueue.name = "REST.\(name).operationQueue"

            self.configuration = configuration
            self.requestFactory = requestFactory
            self.responseDecoder = responseDecoder
        }

        @available(*, deprecated, message: "Use makeRequestExecutor() instead")
        func addOperation<Success>(
            name: String,
            retryStrategy: REST.RetryStrategy,
            requestHandler: RESTRequestHandler,
            responseHandler: some RESTResponseHandler<Success>,
            completionHandler: @escaping CompletionHandler<Success>
        ) -> Cancellable {
            let executor = makeRequestExecutor(
                name: name,
                requestHandler: requestHandler,
                responseHandler: responseHandler
            )

            return executor.execute(retryStrategy: retryStrategy, completionHandler: completionHandler)
        }

        func makeRequestExecutor<Success>(
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
    private struct NetworkOperationFactory<Success, ConfigurationType: ProxyConfiguration> {
        let dispatchQueue: DispatchQueue
        let configuration: ConfigurationType

        let name: String
        let requestHandler: RESTRequestHandler
        let responseHandler: any RESTResponseHandler<Success>

        /// Creates new network operation but does not schedule it for execution.
        func makeOperation(
            retryStrategy: REST.RetryStrategy,
            completionHandler: ((Result<Success, Swift.Error>) -> Void)? = nil
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

    /// Network request executor that supports either callback or async based execution flows.
    private struct RequestExecutor<Success, ConfigurationType: ProxyConfiguration>: RESTRequestExecutor {
        let operationFactory: NetworkOperationFactory<Success, ConfigurationType>
        let operationQueue: AsyncOperationQueue

        func execute(
            retryStrategy: REST.RetryStrategy,
            completionHandler: @escaping (Result<Success, Swift.Error>) -> Void
        ) -> Cancellable {
            let operation = operationFactory.makeOperation(
                retryStrategy: retryStrategy,
                completionHandler: completionHandler
            )

            operationQueue.addOperation(operation)

            return operation
        }

        func execute(retryStrategy: REST.RetryStrategy) async throws -> Success {
            let operation = operationFactory.makeOperation(retryStrategy: retryStrategy, completionHandler: nil)

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

        func execute(completionHandler: @escaping (Result<Success, Swift.Error>) -> Void) -> Cancellable {
            execute(retryStrategy: .noRetry, completionHandler: completionHandler)
        }

        func execute() async throws -> Success {
            return try await execute(retryStrategy: .noRetry)
        }
    }

    public class ProxyConfiguration {
        public let transportProvider: () -> RESTTransport?
        public let addressCacheStore: AddressCache

        public init(
            transportProvider: @escaping () -> RESTTransport?,
            addressCacheStore: AddressCache
        ) {
            self.transportProvider = transportProvider
            self.addressCacheStore = addressCacheStore
        }
    }

    public class AuthProxyConfiguration: ProxyConfiguration {
        public let accessTokenManager: AccessTokenManager

        public init(
            proxyConfiguration: ProxyConfiguration,
            accessTokenManager: AccessTokenManager
        ) {
            self.accessTokenManager = accessTokenManager

            super.init(
                transportProvider: proxyConfiguration.transportProvider,
                addressCacheStore: proxyConfiguration.addressCacheStore
            )
        }
    }
}
