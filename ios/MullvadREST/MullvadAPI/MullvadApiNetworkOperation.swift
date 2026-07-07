//
//  MullvadApiNetworkOperation.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2025-01-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadRustRuntime
import MullvadTypes
import Operations

private enum MullvadApiTransportError: Error {
    case connectionFailed(description: String?)
}

extension REST {
    class MullvadApiNetworkOperation<Success: Sendable>: ResultOperation<Success>, @unchecked Sendable {
        private let logger: Logger

        private let request: APIRequest
        private let transportProvider: APITransportProviderProtocol
        private var responseDecoder: JSONDecoder
        private let responseHandler: any RESTRustResponseHandler<Success>
        private var networkTask: Cancellable?

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            request: APIRequest,
            transportProvider: APITransportProviderProtocol,
            responseDecoder: JSONDecoder,
            responseHandler: some RESTRustResponseHandler<Success>,
            completionHandler: CompletionHandler? = nil
        ) {
            self.request = request
            self.transportProvider = transportProvider
            self.responseDecoder = responseDecoder
            self.responseHandler = responseHandler

            var logger = Logger(label: "REST.RustNetworkOperation")

            logger[metadataKey: "name"] = .string(name)
            self.logger = logger

            super.init(
                dispatchQueue: dispatchQueue,
                completionQueue: .main,
                completionHandler: completionHandler
            )
        }

        override public func operationDidCancel() {
            networkTask?.cancel()
            networkTask = nil
        }

        override func execute() async throws -> Success {
            try await startRequest()
        }

        private func startRequest() async throws -> Success {

            try Task.checkCancellation()

            let transport = transportProvider.makeTransport()

            logger.info("\(#function): using transport=\(transport?.name ?? "Unknown")")

            return try await withCheckedThrowingContinuation { continuation in
                do {
                    networkTask = try transport?.sendRequest(request) { [weak self] response in
                        guard let self else {
                            continuation.resume(throwing: CancellationError())
                            return
                        }

                        logger.debug("\(#function): \(request.name) API response received")

                        if let apiError = response.error {
                            logger.error(
                                "Response contained error code \(apiError.statusCode), error: \(apiError.errorDescription)"
                            )

                            continuation.resume(
                                throwing: restError(apiError: apiError)
                            )
                            return
                        }

                        switch responseHandler.handleResponse(response) {

                        case .success(let value):
                            logger.debug("API response decoded successfully")
                            continuation.resume(returning: value)

                        case .decoding(let block):
                            do {
                                let value = try block()
                                logger.debug("API response decoded via block")
                                continuation.resume(returning: value)
                            } catch {
                                logger.error("Response decoding failed error=\(error)")
                                continuation.resume(
                                    throwing: REST.Error.unhandledResponse(0, nil)
                                )
                            }

                        case .unhandledResponse(let error):
                            logger.error("Unhandled API response error=\(String(describing: error))")
                            continuation.resume(
                                throwing: REST.Error.unhandledResponse(0, error)
                            )
                        }
                    }

                } catch {
                    logger.error("Request failed to send error=\(error)")
                    continuation.resume(throwing: error)
                }
            }
        }

        private func restError(apiError: APIError) -> Error {
            guard let serverResponseCode = apiError.serverResponseCode else {
                return .transport(MullvadApiTransportError.connectionFailed(description: apiError.errorDescription))
            }

            let response = REST.ServerErrorResponse(
                code: REST.ServerResponseCode(rawValue: serverResponseCode),
                detail: apiError.errorDescription
            )
            return .unhandledResponse(apiError.statusCode, response)
        }
    }
}
