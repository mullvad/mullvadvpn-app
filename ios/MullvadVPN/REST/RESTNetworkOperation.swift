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

        private let dispatchQueue: DispatchQueue
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
        )
        {
            self.dispatchQueue = dispatchQueue
            self.urlSession = configuration.session
            self.addressCacheStore = configuration.addressCacheStore
            self.retryStrategy = retryStrategy
            self.requestHandler = requestHandler
            self.responseHandler = responseHandler

            var logger = Logger(label: "REST.NetworkOperation")
            logger[metadataKey: "name"] =  .string(name)
            self.logger = logger

            super.init(completionQueue: .main, completionHandler: completionHandler)
        }

        override func cancel() {
            super.cancel()

            dispatchQueue.async {
                self.retryTimer?.cancel()
                self.networkTask?.cancel()
                self.authorizationTask?.cancel()

                self.retryTimer = nil
                self.networkTask = nil
                self.authorizationTask = nil
            }
        }

        override func main() {
            dispatchQueue.async {
                self.startRequest()
            }
        }

        private func startRequest() {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(completion: .cancelled)
                return
            }

            let authorizationResult = requestHandler.requestAuthorization { completion in
                self.dispatchQueue.async {
                    assert(self.requiresAuthorization, "Illegal use of completion handler.")

                    switch completion {
                    case .success(let authorization):
                        self.didReceiveAuthorization(authorization)

                    case .failure(let error):
                        self.didFailToRequestAuthorization(error)

                    case .cancelled:
                        self.finish(completion: .cancelled)
                    }
                }
            }

            switch authorizationResult {
            case .pending(let task):
                requiresAuthorization = true
                authorizationTask = task

            case .noRequirement:
                requiresAuthorization = false
                didReceiveAuthorization(nil)
            }
        }

        private func didReceiveAuthorization(_ authorization: REST.Authorization?) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(completion: .cancelled)
                return
            }

            let endpoint = self.addressCacheStore.getCurrentEndpoint()

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
                chainedError: error,
                message: "Failed to request authorization."
            )

            finish(completion: .failure(error))
        }

        private func didReceiveURLRequest(_ restRequest: REST.Request, endpoint: AnyIPEndpoint) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger.debug("Send request to \(restRequest.pathTemplate.templateString) via \(endpoint).")

            networkTask = urlSession.dataTask(with: restRequest.urlRequest) { [weak self] data, response, error in
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
                chainedError: error,
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
                chainedError: AnyChainedError(urlError),
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

        private func didReceiveURLResponse(_ response: HTTPURLResponse, data: Data, endpoint: AnyIPEndpoint) {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            logger.debug("Response: \(response.statusCode).")

            let handlerResult = responseHandler.handleURLResponse(response, data: data)

            switch handlerResult {
            case .success(let output):
                // Response handler produced value.
                finish(completion: .success(output))

            case .decoding(let decoderBlock):
                // Response handler returned a block decoding value.
                let decodeResult = Result { try decoderBlock() }
                    .mapError { error -> REST.Error in
                        return .decodeResponse(error)
                    }
                finish(completion: OperationCompletion(result: decodeResult))

            case .unhandledResponse(let serverErrorResponse):
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
