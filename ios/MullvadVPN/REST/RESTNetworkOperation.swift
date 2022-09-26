//
//  NetworkOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

extension REST {
    class NetworkOperation<Success>: ResultOperation<Success, REST.Error> {
        private let requestHandler: AnyRequestHandler
        private let responseHandler: AnyResponseHandler<Success>

        private let logger: Logger
        private let urlSession: URLSession
        private let addressCacheStore: AddressCache.Store

        private var networkTask: URLSessionTask?
        private var authorizationTask: Cancellable?

        private var requiresAuthorization = false
        private var retryInvalidAccessTokenError = true

        private let retryStrategy: RetryStrategy
        private var retryTimer: DispatchSourceTimer?
        private var retryCount = 0

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            configuration: ProxyConfiguration,
            retryStrategy: RetryStrategy,
            requestHandler: AnyRequestHandler,
            responseHandler: AnyResponseHandler<Success>,
            completionHandler: @escaping CompletionHandler
        ) {
            urlSession = configuration.session
            addressCacheStore = configuration.addressCacheStore
            self.retryStrategy = retryStrategy
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

        override func operationDidCancel() {
            retryTimer?.cancel()
            networkTask?.cancel()
            authorizationTask?.cancel()

            retryTimer = nil
            networkTask = nil
            authorizationTask = nil
        }

        override func main() {
            startRequest()
        }

        private func startRequest() {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(completion: .cancelled)
                return
            }

            guard let authorizationProvider = requestHandler.authorizationProvider else {
                requiresAuthorization = false
                didReceiveAuthorization(nil)
                return
            }

            requiresAuthorization = true
            authorizationTask = authorizationProvider.getAuthorization { completion in
                self.dispatchQueue.async {
                    switch completion {
                    case let .success(authorization):
                        self.didReceiveAuthorization(authorization)

                    case let .failure(error):
                        self.didFailToRequestAuthorization(error)

                    case .cancelled:
                        self.finish(completion: .cancelled)
                    }
                }
            }
        }

        private func didReceiveAuthorization(_ authorization: REST.Authorization?) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(completion: .cancelled)
                return
            }

            let endpoint = addressCacheStore.getCurrentEndpoint()

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

        private func didFailToRequestAuthorization(_ error: REST.Error) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger.error(
                error: error,
                message: "Failed to request authorization."
            )

            finish(completion: .failure(error))
        }

        private func didReceiveURLRequest(_ restRequest: REST.Request, endpoint: AnyIPEndpoint) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger
                .debug(
                    "Send request to \(restRequest.pathTemplate.templateString) via \(endpoint)."
                )

            networkTask = urlSession
                .dataTask(with: restRequest.urlRequest) { [weak self] data, response, error in
                    guard let self = self else { return }

                    self.dispatchQueue.async {
                        if let error = error {
                            let urlError = error as! URLError

                            self.didReceiveURLError(urlError, endpoint: endpoint)
                        } else {
                            let httpResponse = response as! HTTPURLResponse
                            let data = data ?? Data()

                            self.didReceiveURLResponse(httpResponse, data: data, endpoint: endpoint)
                        }
                    }
                }

            networkTask?.resume()
        }

        private func didFailToCreateURLRequest(_ error: REST.Error) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger.error(
                error: error,
                message: "Failed to create URLRequest."
            )

            finish(completion: .failure(error))
        }

        private func didReceiveURLError(_ urlError: URLError, endpoint: AnyIPEndpoint) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            switch urlError.code {
            case .cancelled:
                finish(completion: .cancelled)
                return

            case .notConnectedToInternet, .internationalRoamingOff, .callIsActive:
                break

            default:
                _ = addressCacheStore.selectNextEndpoint(endpoint)
            }

            logger.error(
                error: urlError,
                message: "Failed to perform request to \(endpoint)."
            )

            // Check if retry count is not exceeded.
            guard retryCount < retryStrategy.maxRetryCount else {
                if retryStrategy.maxRetryCount > 0 {
                    logger.debug("Ran out of retry attempts (\(retryStrategy.maxRetryCount))")
                }

                finish(completion: OperationCompletion(result: .failure(.network(urlError))))
                return
            }

            // Increment retry count.
            retryCount += 1

            // Retry immediatly if retry delay is set to never.
            guard retryStrategy.retryDelay != .never else {
                startRequest()
                return
            }

            // Create timer to delay retry.
            let timer = DispatchSource.makeTimerSource(queue: dispatchQueue)

            timer.setEventHandler { [weak self] in
                self?.startRequest()
            }

            timer.setCancelHandler { [weak self] in
                self?.finish(completion: .cancelled)
            }

            timer.schedule(wallDeadline: .now() + retryStrategy.retryDelay)
            timer.activate()

            retryTimer = timer
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
                finish(completion: .success(output))

            case let .decoding(decoderBlock):
                // Response handler returned a block decoding value.
                let decodeResult = Result { try decoderBlock() }
                    .mapError { error -> REST.Error in
                        return .decodeResponse(error)
                    }
                finish(completion: OperationCompletion(result: decodeResult))

            case let .unhandledResponse(serverErrorResponse):
                // Response handler couldn't handle the response.
                if serverErrorResponse?.code == .invalidAccessToken,
                   requiresAuthorization,
                   retryInvalidAccessTokenError
                {
                    logger.debug("Received invalid access token error. Retry once.")
                    retryInvalidAccessTokenError = false
                    startRequest()
                } else {
                    finish(
                        completion: .failure(
                            .unhandledResponse(response.statusCode, serverErrorResponse)
                        )
                    )
                }
            }
        }
    }
}
