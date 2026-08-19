//
//  MullvadApiNetworkTask.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-08-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadRustRuntime
import MullvadTypes

private enum MullvadApiTransportError: Error {
    case connectionFailed(description: String?)
}

extension REST {
    class MullvadApiNetworkTask<Success: Sendable> {
        private let logger: Logger

        private let request: APIRequest
        private let transportProvider: APITransportProviderProtocol
        private let responseHandler: any RESTRustResponseHandler<Success>

        init(
            name: String,
            request: APIRequest,
            transportProvider: APITransportProviderProtocol,
            responseHandler: some RESTRustResponseHandler<Success>,
        ) {
            self.request = request
            self.transportProvider = transportProvider
            self.responseHandler = responseHandler

            var logger = Logger(label: "REST.RustNetworkOperation")
            logger[metadataKey: "name"] = .string(name)
            self.logger = logger
        }

        func startRequest() async -> Result<Success, Swift.Error> {
            guard !Task.isCancelled else {
                return .failure(CancellationError())
            }

            guard let transport = transportProvider.makeTransport() else {
                return .failure(InternalTransportError.noTransport)
            }

            do {
                logger.info("\(#function): using transport=\(transport.name)")

                let response = try await transport.sendRequest(request)

                logger.debug("\(#function): \(request.name) API response received")

                if let apiError = response.error {
                    logger
                        .error(
                            "Response contained error code \(apiError.statusCode), error: \(apiError.errorDescription)"
                        )
                    return .failure(restError(apiError: apiError))
                }

                let decodedResponse = responseHandler.handleResponse(response)

                switch decodedResponse {
                case let .success(value):
                    logger.debug("API response decoded successfully")
                    return .success(value)
                case let .decoding(block):
                    do {
                        let value = try block()
                        logger.debug("API response decoded via block")
                        return .success(value)
                    } catch {
                        logger.error("Response decoding failed error=\(error)")
                        return .failure(REST.Error.unhandledResponse(0, nil))
                    }
                case let .unhandledResponse(error):
                    logger.error("Unhandled API response error=\(String(describing: error))")
                    return .failure(REST.Error.unhandledResponse(0, error))
                }
            } catch {
                logger.error("Request failed to send error=\(error)")
                return .failure(error)
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
