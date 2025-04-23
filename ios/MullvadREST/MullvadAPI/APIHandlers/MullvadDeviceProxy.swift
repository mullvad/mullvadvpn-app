//
//  MullvadDeviceProxy.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-04-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadRustRuntime
import MullvadTypes
import Operations
import WireGuardKitTypes

extension REST {
    final class MullvadDeviceProxy: DeviceHandling, @unchecked Sendable {
        let transportProvider: APITransportProviderProtocol
        let dispatchQueue: DispatchQueue
        let operationQueue = AsyncOperationQueue()
        let responseDecoder: JSONDecoder

        public init(
            transportProvider: APITransportProviderProtocol,
            dispatchQueue: DispatchQueue,
            responseDecoder: JSONDecoder
        ) {
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
            let responseHandler = rustResponseHandler(
                decoding: Device.self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .getDevice(retryStrategy, accountNumber: accountNumber, identifier: identifier),
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        func getDevices(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<[Device]>
        ) -> Cancellable {
            let responseHandler = rustResponseHandler(
                decoding: [Device].self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .getDevices(retryStrategy, accountNumber: accountNumber),
                responseHandler: responseHandler,
                completionHandler: completion
            )
        }

        func createDevice(
            accountNumber: String,
            request: CreateDeviceRequest,
            retryStrategy: REST.RetryStrategy,
            completion: @escaping ProxyCompletionHandler<Device>
        ) -> Cancellable {
            let responseHandler = rustResponseHandler(
                decoding: Device.self,
                with: responseDecoder
            )

            return createNetworkOperation(
                request: .createDevice(retryStrategy, accountNumber: accountNumber, request: request),
                responseHandler: responseHandler,
                completionHandler: completion
            )
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

        func rotateDeviceKey(
            accountNumber: String,
            identifier: String,
            publicKey: PublicKey,
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
    }
}
