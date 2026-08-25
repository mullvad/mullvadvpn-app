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
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Device, Swift.Error> {
            let request = APIRequest.getDevice(retryStrategy, accountNumber: accountNumber, identifier: identifier)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: Device.self,
                    with: responseDecoder
                )
            )

            return await task.startRequest()
        }

        func getDevices(
            accountNumber: String,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<[Device], Swift.Error> {
            let request = APIRequest.getDevices(retryStrategy, accountNumber: accountNumber)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: [Device].self,
                    with: responseDecoder
                )
            )

            return await task.startRequest()
        }

        func createDevice(
            accountNumber: String,
            request: CreateDeviceRequest,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Device, Swift.Error> {
            let request = APIRequest.createDevice(retryStrategy, accountNumber: accountNumber, request: request)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: Device.self,
                    with: responseDecoder
                )
            )

            return await task.startRequest()
        }

        func deleteDevice(
            accountNumber: String,
            identifier: String,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Bool, Swift.Error> {
            let request = APIRequest.deleteDevice(retryStrategy, accountNumber: accountNumber, identifier: identifier)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustEmptyResponseHandler()
            )

            let result = await task.startRequest()
            return switch result {
            case .success(let success):
                .success(true)
            case .failure(let failure):
                .failure(failure)
            }
        }

        func rotateDeviceKey(
            accountNumber: String,
            identifier: String,
            publicKey: WireGuard.PublicKey,
            retryStrategy: REST.RetryStrategy
        ) async -> Result<Device, Swift.Error> {
            let request = APIRequest.rotateDeviceKey(
                retryStrategy, accountNumber: accountNumber, identifier: identifier, publicKey: publicKey)

            let task = MullvadApiNetworkTask(
                name: request.name,
                request: request,
                transportProvider: transportProvider,
                responseHandler: rustResponseHandler(
                    decoding: Device.self,
                    with: responseDecoder
                )
            )
            return await task.startRequest()
        }
    }
}
