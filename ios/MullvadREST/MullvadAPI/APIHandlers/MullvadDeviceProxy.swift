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
            Task {
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
            let data = try response.get().body
            return try responseDecoder.decode(Device.self, from: data)
        }

        func getDevices(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<[Device]>
        ) -> Cancellable {
            Task {
                do {
                    let devices = try await getDevices(
                        accountNumber: accountNumber,
                        retryStrategy: retryStrategy)
                    completion(.success(devices))
                } catch {
                    completion(.failure(error))
                }
            }.cancellable
        }

        func getDevices(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) async throws -> [Device] {
            let response = try await apiContext.getDevices(
                retryStrategy: retryStrategy.toRustStrategy(),
                accountNumber: accountNumber)
            let body = try response.get().body
            let devices = try responseDecoder.decode([Device].self, from: body)

            return devices
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
            let body = try response.get().body
            let device = try responseDecoder.decode(Device.self, from: body)

            return device
        }

        func deleteDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Bool>
        ) -> Cancellable {
            let responseHandler = rustEmptyResponseHandler()

            return createNetworkOperation(
                request: .deleteDevice(retryStrategy, accountNumber: accountNumber, identifier: identifier),
                responseHandler: responseHandler
            ) { result in
                if case let .failure(err) = result {
                    completion(.failure(err))
                } else {
                    completion(.success(true))
                }
            }
        }

        func deleteDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy
        ) async throws -> Bool {
            try await executeRequest(
                .deleteDevice(
                    retryStrategy,
                    accountNumber: accountNumber,
                    identifier: identifier
                )
            )
        }

        func rotateDeviceKey(
            accountNumber: String,
            identifier: String,
            publicKey: WireGuard.PublicKey,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Device>
        ) -> Cancellable {
            let responseHandler = rustResponseHandler(
                decoding: Device.self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .rotateDeviceKey(
                    retryStrategy,
                    accountNumber: accountNumber,
                    identifier: identifier,
                    publicKey: publicKey
                ),
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        func rotateDeviceKey(
            accountNumber: String,
            identifier: String,
            publicKey: WireGuard.PublicKey,
            retryStrategy: REST.RetryStrategy,
        ) async throws -> Device {
            try await executeRequest(
                .rotateDeviceKey(
                    retryStrategy,
                    accountNumber: accountNumber,
                    identifier: identifier,
                    publicKey: publicKey
                )
            )
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
