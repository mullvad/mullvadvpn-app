//
//  Tunnel+Messaging.swift
//  MullvadVPN
//
//  Created by pronebird on 16/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Operations

/// Shared operation queue used for IPC requests.
private let operationQueue = AsyncOperationQueue()

/// Shared queue used by IPC operations.
private let dispatchQueue = DispatchQueue(label: "Tunnel.dispatchQueue")

extension Tunnel {
    /// Request packet tunnel process to reconnect the tunnel with the given relay selector result.
    /// Packet tunnel will reconnect to the current relay if relay selector result is not provided.
    func reconnectTunnel(
        relaySelectorResult: RelaySelectorResult?,
        completionHandler: @escaping (OperationCompletion<Void, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            tunnel: self,
            message: .reconnectTunnel(relaySelectorResult),
            completionHandler: completionHandler
        )

        operationQueue.addOperation(operation)

        return operation
    }

    /// Request status from packet tunnel process.
    func getTunnelStatus(
        completionHandler: @escaping (OperationCompletion<PacketTunnelStatus, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            tunnel: self,
            message: .getTunnelStatus,
            completionHandler: completionHandler
        )

        operationQueue.addOperation(operation)

        return operation
    }

    /// Request packet tunnel to transport a http request.
    /// - Parameters:
    ///   - requestData: Serialized data to be regenerated for URLSession inside tunnel.
    ///   - completionHandler: Packet tunnel reply with OperationCompletion.
    /// - Returns: Cancellable.
    func sendRequest(
        _ requestData: ProxyURLRequest,
        completionHandler: @escaping (OperationCompletion<ProxyURLResponse, Error>) -> Void
    ) -> Cancellable {
        let operation = SendTunnelProviderMessageOperation(
            dispatchQueue: dispatchQueue,
            tunnel: self,
            message: .sendURLRequest(requestData),
            timeout: 12,
            completionHandler: completionHandler
        )

        operation.addBlockObserver(
            OperationBlockObserver(didCancel: { _ in
                let cancelOperation = SendTunnelProviderMessageOperation(
                    dispatchQueue: dispatchQueue,
                    tunnel: self,
                    message: .cancelURLRequest(requestData.id),
                    completionHandler: nil
                )

                operationQueue.addOperation(cancelOperation)
            })
        )

        operationQueue.addOperation(operation)

        return operation
    }
}
