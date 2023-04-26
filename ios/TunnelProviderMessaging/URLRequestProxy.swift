//
//  URLRequestProxy.swift
//  PacketTunnel
//
//  Created by pronebird on 03/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

public final class URLRequestProxy {
    /// Serial queue used for synchronizing access to class members.
    private let dispatchQueue: DispatchQueue

    private let transportProvider: () -> RESTTransportProvider?

    /// List of all proxied network requests bypassing VPN.
    private var proxiedRequests: [UUID: Cancellable] = [:]

    public init(
        dispatchQueue: DispatchQueue,
        transportProvider: @escaping () -> RESTTransportProvider?
    ) {
        self.dispatchQueue = dispatchQueue
        self.transportProvider = transportProvider
    }

    public func sendRequest(
        _ proxyRequest: ProxyURLRequest,
        completionHandler: @escaping (ProxyURLResponse) -> Void
    ) {
        // Instruct the Packet Tunnel to try to reach the API via the local shadow socks proxy instance if needed
        let transportProvider = transportProvider()
        if proxyRequest.useShadowsocksTransport {
            transportProvider?.selectNextTransport()
        }

        guard let transport = transportProvider?.transport() else { return }
        // The task sent by `transport.sendRequest` comes in an already resumed state
        let task = transport.sendRequest(proxyRequest.urlRequest) { [weak self] data, response, error in
            guard let self = self else { return }
            let response = ProxyURLResponse(data: data, response: response, error: error)

            _ = removeRequest(identifier: proxyRequest.id)

            completionHandler(response)
        }
        dispatchQueue.async { [weak self] in
            guard let self = self else { return }
            // All tasks should have unique identifiers, but if not, cancel the task scheduled
            // earlier.
            let oldTask = addRequest(identifier: proxyRequest.id, task: task)
            oldTask?.cancel()
        }
    }

    public func cancelRequest(identifier: UUID) {
        dispatchQueue.async {
            let task = self.removeRequest(identifier: identifier)

            task?.cancel()
        }
    }

    private func addRequest(identifier: UUID, task: Cancellable) -> Cancellable? {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))
        return proxiedRequests.updateValue(task, forKey: identifier)
    }

    private func removeRequest(identifier: UUID) -> Cancellable? {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))
        return proxiedRequests.removeValue(forKey: identifier)
    }
}
