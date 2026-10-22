// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
import MullvadRustRuntime
import MullvadTypes
import Operations

extension REST {
    final class MullvadDeviceProxy: DeviceHandling, @unchecked Sendable {
        let apiContext: ApiContext
        let transportProvider: APITransportProviderProtocol
        let dispatchQueue: DispatchQueue
        let operationQueue = AsyncOperationQueue()
        let responseDecoder: JSONDecoder

        public init(
            apiContext: ApiContext,
            transportProvider: APITransportProviderProtocol,
            dispatchQueue: DispatchQueue,
            responseDecoder: JSONDecoder
        ) {
            self.apiContext = apiContext
            self.transportProvider = transportProvider
            self.dispatchQueue = dispatchQueue
            self.responseDecoder = responseDecoder
        }

        func getDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Device>
        ) -> Cancellable {
            return Task {
                do {
                    let device = try await getDevice(
                        accountNumber: accountNumber,
                        identifier: identifier,
                        retryStrategy: retryStrategy)
                    completion(.success(device))
                } catch {
                    completion(.failure(error))
                }
            }.cancellable
        }

        func getDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy
        ) async throws -> Device {
            let response = try await apiContext.getDevice(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber,
                identifier: identifier)
            return try decode(
                decoding: Device.self,
                responseDecoder: responseDecoder,
                response: response)
        }

        func getDevices(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
        ) async throws -> [Device] {
            let response = try await apiContext.getDevices(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber)

            return try decode(
                decoding: [Device].self,
                responseDecoder: responseDecoder,
                response: response)
        }

        func createDevice(
            accountNumber: String,
            request: CreateDeviceRequest,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Device>
        ) -> Cancellable {
            Task {
                do {
                    let device = try await createDevice(
                        accountNumber: accountNumber,
                        request: request,
                        retryStrategy: retryStrategy)
                    completion(.success(device))
                } catch {
                    completion(.failure(error))
                }
            }.cancellable
        }

        func createDevice(
            accountNumber: String,
            request: CreateDeviceRequest,
            retryStrategy: REST.RetryStrategy
        ) async throws -> Device {
            let response = try await apiContext.createDevice(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber,
                publicKey: request.publicKey.rawValue)
            return try decode(
                decoding: Device.self,
                responseDecoder: responseDecoder,
                response: response)
        }

        func deleteDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Bool>
        ) -> Cancellable {
            Task {
                do {
                    let result = try await deleteDevice(
                        accountNumber: accountNumber,
                        identifier: identifier,
                        retryStrategy: retryStrategy)
                    completion(.success(result))
                } catch {
                    completion(.failure(error))
                }
            }.cancellable
        }

        func deleteDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy
        ) async throws -> Bool {
            let response = try await apiContext.deleteDevice(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber,
                identifier: identifier)
            switch response {
            case .body:
                return true
            case .error(let statusCode, let errorDescription, let serverResponseCode):
                throw APIError(
                    statusCode: Int(statusCode),
                    errorDescription: errorDescription ?? "",
                    serverResponseCode: serverResponseCode)
            }
        }

        func rotateDeviceKey(
            accountNumber: String,
            identifier: String,
            publicKey: WireGuard.PublicKey,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Device>
        ) -> Cancellable {
            Task {
                do {
                    let device = try await rotateDeviceKey(
                        accountNumber: accountNumber,
                        identifier: identifier,
                        publicKey: publicKey,
                        retryStrategy: retryStrategy)
                    completion(.success(device))
                } catch {
                    completion(.failure(error))
                }
            }.cancellable
        }

        func rotateDeviceKey(
            accountNumber: String,
            identifier: String,
            publicKey: WireGuard.PublicKey,
            retryStrategy: REST.RetryStrategy,
        ) async throws -> Device {
            let response = try await apiContext.rotateDeviceKey(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber,
                identifier: identifier,
                publicKey: publicKey.rawValue)
            return try decode(
                decoding: Device.self,
                responseDecoder: responseDecoder,
                response: response)
        }

        private func createNetworkOperation<Success: Any>(
            request: APIRequest,
            responseHandler: RustResponseHandler<Success>,
            completionHandler: @escaping @Sendable ProxyCompletionHandler<Success>
        ) -> MullvadApiNetworkOperation<Success> {
            let networkOperation = MullvadApiNetworkOperation(
                name: request.name,
                dispatchQueue: dispatchQueue,
                request: request,
                transportProvider: transportProvider,
                responseDecoder: responseDecoder,
                responseHandler: responseHandler,
                completionHandler: completionHandler
            )

            operationQueue.addOperation(networkOperation)

            return networkOperation
        }

        private func executeRequest<Success: Sendable & Decodable>(
            _ request: APIRequest
        ) async throws -> Success {
            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: Success.self,
                    with: responseDecoder
                )
            )
            return try await task.startRequest().get()
        }
    }
}
