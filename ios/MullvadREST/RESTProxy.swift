//
//  RESTProxy.swift
//  MullvadVPN
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Operations

extension REST {
    public class Proxy<ConfigurationType: ProxyConfiguration> {
        public typealias CompletionHandler<Success> = (OperationCompletion<Success, REST.Error>)
            -> Void

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
            requestHandler: RESTRequestHandler,
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

    public class ProxyConfiguration {
        public let transportRegistry: REST.TransportRegistry
        public let addressCacheStore: AddressCache.Store

        public init(
            transportRegistry: REST.TransportRegistry,
            addressCacheStore: AddressCache.Store
        ) {
            self.transportRegistry = transportRegistry
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
                transportRegistry: proxyConfiguration.transportRegistry,
                addressCacheStore: proxyConfiguration.addressCacheStore
            )
        }
    }
}
