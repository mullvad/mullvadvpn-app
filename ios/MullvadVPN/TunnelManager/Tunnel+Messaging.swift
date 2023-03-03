//
//  Tunnel+Messaging.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations
import RelaySelector
import TunnelProviderMessaging

/// Shared operation queue used for IPC requests.
private let operationQueue = AsyncOperationQueue()

/// Shared queue used by IPC operations.
private let dispatchQueue = DispatchQueue(label: "Tunnel.dispatchQueue")

/// Timeout for proxy requests.
private let proxyRequestTimeout = REST.defaultAPINetworkTimeout + 2

extension Tunnel {
    /// Request packet tunnel process to reconnect the tunnel with the given relay selector result.
    /// Packet tunnel will reconnect to the current relay if relay selector result is not provided.
    func reconnectTunnel(
        relaySelectorResult: RelaySelectorResult?,
        completionHandler: @escaping (Result<Void, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            application: .shared,
            tunnel: self,
            message: .reconnectTunnel(relaySelectorResult),
            completionHandler: completionHandler
        )

        operationQueue.addOperation(operation)

        return operation
    }

    /// Request status from packet tunnel process.
    func getTunnelStatus(
        completionHandler: @escaping (Result<PacketTunnelStatus, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            application: .shared,
            tunnel: self,
            message: .getTunnelStatus,
            completionHandler: completionHandler
        )

        operationQueue.addOperation(operation)

        return operation
    }

    /// Send HTTP request via packet tunnel process bypassing VPN.
    func sendRequest(
        _ proxyRequest: ProxyURLRequest,
        completionHandler: @escaping (Result<ProxyURLResponse, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            application: .shared,
            tunnel: self,
            message: .sendURLRequest(proxyRequest),
            timeout: proxyRequestTimeout,
            completionHandler: completionHandler
        )

        operation.addBlockObserver(
            OperationBlockObserver(didCancel: { [weak self] _ in
                guard let self = self else { return }

                let cancelOperation = SendTunnelProviderMessageOperation(
                    dispatchQueue: dispatchQueue,
                    application: .shared,
                    tunnel: self,
                    message: .cancelURLRequest(proxyRequest.id),
                    completionHandler: nil
                )

                operationQueue.addOperation(cancelOperation)
            })
        )

        operationQueue.addOperation(operation)

        return operation
    }
}
