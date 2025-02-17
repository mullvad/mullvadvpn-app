//
//  Tunnel+Messaging.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations
import PacketTunnelCore

/// Shared operation queue used for IPC requests.
private let operationQueue = AsyncOperationQueue()

/// Shared queue used by IPC operations.
private let dispatchQueue = DispatchQueue(label: "Tunnel.dispatchQueue")

/// Timeout for proxy requests.
private let proxyRequestTimeout = REST.defaultAPINetworkTimeout + 2

extension TunnelProtocol {
    /// Request packet tunnel process to reconnect the tunnel with the given relays.
    func reconnectTunnel(
        to nextRelays: NextRelays,
        completionHandler: @escaping @Sendable (Result<Void, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            backgroundTaskProvider: backgroundTaskProvider,
            tunnel: self,
            message: .reconnectTunnel(nextRelays),
            decoderHandler: { _ in () },
            completionHandler: completionHandler
        )

        operationQueue.addOperation(operation)

        return operation
    }

    /// Request status from packet tunnel process.
    func getTunnelStatus(
        completionHandler: @escaping @Sendable (Result<ObservedState, Error>) -> Void
    ) -> Cancellable {
        let decoderHandler: (Data?) throws -> ObservedState = { data in
            if let data {
                return try TunnelProviderReply<ObservedState>(messageData: data).value
            } else {
                throw EmptyTunnelProviderResponseError()
            }
        }

        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            backgroundTaskProvider: backgroundTaskProvider,
            tunnel: self,
            message: .getTunnelStatus,
            decoderHandler: decoderHandler,
            completionHandler: completionHandler
        )

        operationQueue.addOperation(operation)
        return operation
    }

    /// Send HTTP request via packet tunnel process bypassing VPN.
    func sendRequest(
        _ proxyRequest: ProxyURLRequest,
        completionHandler: @escaping @Sendable (Result<ProxyURLResponse, Error>) -> Void
    ) -> Cancellable {
        let decoderHandler: (Data?) throws -> ProxyURLResponse = { data in
            if let data {
                return try TunnelProviderReply<ProxyURLResponse>(messageData: data).value
            } else {
                throw EmptyTunnelProviderResponseError()
            }
        }

        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            backgroundTaskProvider: backgroundTaskProvider,
            tunnel: self,
            message: .sendURLRequest(proxyRequest),
            timeout: proxyRequestTimeout,
            decoderHandler: decoderHandler,
            completionHandler: completionHandler
        )

        operation.onCancel { [weak self] _ in
            guard let self else { return }

            let cancelOperation = SendTunnelProviderMessageOperation(
                dispatchQueue: dispatchQueue,
                backgroundTaskProvider: backgroundTaskProvider,
                tunnel: self,
                message: .cancelURLRequest(proxyRequest.id),
                decoderHandler: decoderHandler,
                completionHandler: nil
            )

            operationQueue.addOperation(cancelOperation)
        }

        operationQueue.addOperation(operation)

        return operation
    }

    /// Send API request via packet tunnel process bypassing VPN.
    func sendAPIRequest(
        _ proxyRequest: ProxyAPIRequest,
        completionHandler: @escaping @Sendable (Result<ProxyAPIResponse, Error>) -> Void
    ) -> Cancellable {
        let decoderHandler: (Data?) throws -> ProxyAPIResponse = { data in
            if let data {
                return try TunnelProviderReply<ProxyAPIResponse>(messageData: data).value
            } else {
                throw EmptyTunnelProviderResponseError()
            }
        }

        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            backgroundTaskProvider: backgroundTaskProvider,
            tunnel: self,
            message: .sendAPIRequest(proxyRequest),
            timeout: proxyRequestTimeout,
            decoderHandler: decoderHandler,
            completionHandler: completionHandler
        )

        operation.onCancel { [weak self] _ in
            guard let self else { return }

            let cancelOperation = SendTunnelProviderMessageOperation(
                dispatchQueue: dispatchQueue,
                backgroundTaskProvider: backgroundTaskProvider,
                tunnel: self,
                message: .cancelAPIRequest(proxyRequest.id),
                decoderHandler: decoderHandler,
                completionHandler: nil
            )

            operationQueue.addOperation(cancelOperation)
        }

        operationQueue.addOperation(operation)

        return operation
    }

    /// Notify tunnel about private key rotation.
    func notifyKeyRotation(
        completionHandler: @escaping @Sendable (Result<Void, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            backgroundTaskProvider: backgroundTaskProvider,
            tunnel: self,
            message: .privateKeyRotation,
            decoderHandler: { _ in () },
            completionHandler: completionHandler
        )

        operationQueue.addOperation(operation)

        return operation
    }
}
