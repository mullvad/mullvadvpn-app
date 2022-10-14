//
//  RESTProxy.swift
//  MullvadVPN
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Operations

public extension REST {
    class Proxy<ConfigurationType: ProxyConfiguration> {
        public typealias CompletionHandler<Success> = (OperationCompletion<Success, REST.Error>) -> Void

        /// Synchronization queue used by network operations.
        public let dispatchQueue: DispatchQueue

        /// Operation queue used for running network operations.
        public let operationQueue = AsyncOperationQueue()

        /// Proxy configuration.
        public let configuration: ConfigurationType

        /// URL request factory.
        public let requestFactory: REST.RequestFactory

        /// URL response decoder.
        public let responseDecoder: JSONDecoder

        public init(
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

        public func addOperation<Success>(
            name: String,
            retryStrategy: REST.RetryStrategy,
            requestHandler: REST.AnyRequestHandler,
            responseHandler: REST.AnyResponseHandler<Success>,
            completionHandler: @escaping NetworkOperation<Success>.CompletionHandler
        ) -> Cancellable {
            let operation = NetworkOperation(
                name: getTaskIdentifier(name: name),
                dispatchQueue: dispatchQueue,
                configuration: configuration,
                retryStrategy: retryStrategy,
                requestHandler: requestHandler,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )

            operationQueue.addOperation(operation)

            return operation
        }
    }

    class ProxyConfiguration {
        public let transportRegistry: RESTTransportRegistry
        public let addressCacheStore: AddressCache.Store

        public init(transportRegistry: RESTTransportRegistry, addressCacheStore: AddressCache.Store) {
            self.transportRegistry = transportRegistry
            self.addressCacheStore = addressCacheStore
        }
    }

    class AuthProxyConfiguration: ProxyConfiguration {
        public let accessTokenManager: AccessTokenManager

        public init(proxyConfiguration: ProxyConfiguration, accessTokenManager: AccessTokenManager) {
            self.accessTokenManager = accessTokenManager

            super.init(
                transportRegistry: proxyConfiguration.transportRegistry,
                addressCacheStore: proxyConfiguration.addressCacheStore
            )
        }
    }
}
