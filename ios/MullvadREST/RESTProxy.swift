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

        func addOperation<Success>(
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
        public let transportProvider: () -> RESTTransportProvider?
        public let addressCacheStore: AddressCache

        public init(
            transportProvider: @escaping () -> RESTTransportProvider?,
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
