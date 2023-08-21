//
//  NetworkOperation.swift
//  MullvadREST
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import Operations

extension REST {
    class NetworkOperation<Success>: ResultOperation<Success> {
        private let requestHandler: RESTRequestHandler
        private let responseHandler: any RESTResponseHandler<Success>

        private let logger: Logger
        private let transportProvider: () -> RESTTransport?
        private let addressCacheStore: AddressCache

        private var networkTask: Cancellable?
        private var authorizationTask: Cancellable?

        private var requiresAuthorization = false
        private var retryInvalidAccessTokenError = true

        private let retryStrategy: RetryStrategy
        private var retryDelayIterator: AnyIterator<Duration>
        private var retryTimer: DispatchSourceTimer?
        private var retryCount = 0

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            configuration: ProxyConfiguration,
            retryStrategy: RetryStrategy,
            requestHandler: RESTRequestHandler,
            responseHandler: some RESTResponseHandler<Success>,
            completionHandler: CompletionHandler?
        ) {
            addressCacheStore = configuration.addressCacheStore
            transportProvider = configuration.transportProvider
            self.retryStrategy = retryStrategy
            retryDelayIterator = retryStrategy.makeDelayIterator()
            self.requestHandler = requestHandler
            self.responseHandler = responseHandler

            var logger = Logger(label: "REST.NetworkOperation")
            logger[metadataKey: "name"] = .string(name)
            self.logger = logger

            super.init(
                dispatchQueue: dispatchQueue,
                completionQueue: .main,
                completionHandler: completionHandler
            )
        }

        override public func operationDidCancel() {
            retryTimer?.cancel()
            networkTask?.cancel()
            authorizationTask?.cancel()

            retryTimer = nil
            networkTask = nil
            authorizationTask = nil
        }

        override public func main() {
            startRequest()
        }

        private func startRequest() {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(result: .failure(OperationError.cancelled))
                return
            }

            guard let authorizationProvider = requestHandler.authorizationProvider else {
                requiresAuthorization = false
                didReceiveAuthorization(nil)
                return
            }

            requiresAuthorization = true
            authorizationTask = authorizationProvider.getAuthorization { result in
                self.dispatchQueue.async {
                    switch result {
                    case let .success(authorization):
                        self.didReceiveAuthorization(authorization)

                    case let .failure(error):
                        if error.isOperationCancellationError {
                            self.finish(result: .failure(error))
                        } else {
                            self.didFailToRequestAuthorization(error)
                        }
                    }
                }
            }
        }

        private func didReceiveAuthorization(_ authorization: REST.Authorization?) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(result: .failure(OperationError.cancelled))
                return
            }

            let endpoint = REST.isStagingEnvironment ? REST.defaultAPIEndpoint : addressCacheStore.getCurrentEndpoint()

            do {
                let request = try requestHandler.createURLRequest(
                    endpoint: endpoint,
                    authorization: authorization
                )

                didReceiveURLRequest(request, endpoint: endpoint)
            } catch {
                didFailToCreateURLRequest(.createURLRequest(error))
            }
        }

        private func didFailToRequestAuthorization(_ error: Swift.Error) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger.error(
                error: error,
                message: "Failed to request authorization."
            )

            finish(result: .failure(error))
        }

        private func didReceiveURLRequest(_ restRequest: REST.Request, endpoint: AnyIPEndpoint) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            let transport = transportProvider()
            guard let transport else {
                logger.error("Failed to obtain transport.")
                finish(result: .failure(REST.Error.transport(NoTransportError())))
                return
            }

            logger.debug(
                """
                Send request to \(restRequest.pathTemplate.templateString) via \(endpoint) \
                using \(transport.name).
                """
            )

            networkTask = transport.sendRequest(restRequest.urlRequest) { [weak self] data, response, error in
                guard let self else { return }
                dispatchQueue.async {
                    if let error {
                        self.didReceiveError(
                            error,
                            transport: transport,
                            endpoint: endpoint
                        )
                    } else {
                        let httpResponse = response as! HTTPURLResponse
                        let data = data ?? Data()

                        self.didReceiveURLResponse(
                            httpResponse,
                            data: data,
                            endpoint: endpoint
                        )
                    }
                }
            }
        }

        private func didFailToCreateURLRequest(_ error: REST.Error) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger.error(
                error: error,
                message: "Failed to create URLRequest."
            )

            finish(result: .failure(error))
        }

        private func didReceiveError(
            _ error: Swift.Error,
            transport: RESTTransport,
            endpoint: AnyIPEndpoint
        ) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            if case URLError.cancelled = error {
                finish(result: .failure(OperationError.cancelled))
            } else {
                logger.error(
                    error: error,
                    message: "Failed to perform request to \(endpoint) using \(transport.name)."
                )
                retryRequest(with: error)
            }
        }

        private func didReceiveURLResponse(
            _ response: HTTPURLResponse,
            data: Data,
            endpoint: AnyIPEndpoint
        ) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger.debug("Response: \(response.statusCode).")

            let handlerResult = responseHandler.handleURLResponse(response, data: data)

            switch handlerResult {
            case let .success(output):
                // Response handler produced value.
                finish(result: .success(output))

            case let .decoding(decoderBlock):
                // Response handler returned a block decoding value.
                let decodeResult = Result { try decoderBlock() }
                    .mapError { error -> REST.Error in
                        .decodeResponse(error)
                    }
                finish(result: decodeResult.mapError { $0 })

            case let .unhandledResponse(serverErrorResponse):
                // Response handler couldn't handle the response.
                if serverErrorResponse?.code == .invalidAccessToken,
                   requiresAuthorization,
                   retryInvalidAccessTokenError {
                    logger.debug("Received invalid access token error. Retry once.")
                    retryInvalidAccessTokenError = false
                    startRequest()
                } else {
                    finish(result: .failure(
                        REST.Error.unhandledResponse(response.statusCode, serverErrorResponse)
                    ))
                }
            }
        }

        private func retryRequest(with error: Swift.Error) {
            // Check if retry count is not exceeded.
            guard retryCount < retryStrategy.maxRetryCount else {
                if retryStrategy.maxRetryCount > 0 {
                    logger.debug("Ran out of retry attempts (\(retryStrategy.maxRetryCount))")
                }
                finish(result: .failure(wrapRequestError(error)))
                return
            }

            // Increment retry count.
            retryCount += 1

            // Retry immediately if retry delay is set to never.
            guard retryStrategy.delay != .never else {
                startRequest()
                return
            }

            guard let waitDelay = retryDelayIterator.next() else {
                logger.debug("Retry delay iterator failed to produce next value.")

                finish(result: .failure(wrapRequestError(error)))
                return
            }

            logger.debug("Retry in \(waitDelay.format()).")

            // Create timer to delay retry.
            let timer = DispatchSource.makeTimerSource(queue: dispatchQueue)

            timer.setEventHandler { [weak self] in
                self?.startRequest()
            }

            timer.setCancelHandler { [weak self] in
                self?.finish(result: .failure(OperationError.cancelled))
            }

            timer.schedule(wallDeadline: .now() + waitDelay.timeInterval)
            timer.activate()

            retryTimer = timer
        }

        private func wrapRequestError(_ error: Swift.Error) -> REST.Error {
            if let error = error as? URLError {
                return .network(error)
            } else {
                return .transport(error)
            }
        }
    }
}
