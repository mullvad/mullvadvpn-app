//
//  URLRequestProxy.swift
//  PacketTunnelCore
//
//  Created by pronebird on 03/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

/// Network request proxy capable of passing serializable requests and responses over the given transport provider.
public final class URLRequestProxy: @unchecked Sendable {
    /// Serial queue used for synchronizing access to class members.
    private let dispatchQueue: DispatchQueue

    private let transportProvider: RESTTransportProvider

    /// List of all proxied network requests bypassing VPN.
    private var proxiedRequests: [UUID: Cancellable] = [:]

    public init(
        dispatchQueue: DispatchQueue,
        transportProvider: RESTTransportProvider
    ) {
        self.dispatchQueue = dispatchQueue
        self.transportProvider = transportProvider
    }

    public func sendRequest(
        _ proxyRequest: ProxyURLRequest,
        completionHandler: @escaping @Sendable (ProxyURLResponse) -> Void
    ) {
        dispatchQueue.async {
            guard let transportProvider = self.transportProvider.makeTransport() else {
                // Edge case in which case we return `ProxyURLResponse` with no data.
                completionHandler(ProxyURLResponse(data: nil, response: nil, error: nil))
                return
            }

            // The task sent by `transport.sendRequest` comes in an already resumed state
            let task = transportProvider.sendRequest(proxyRequest.urlRequest) { [self] data, response, error in
                // However there is no guarantee about which queue the execution resumes on
                // Use `dispatchQueue` to guarantee thread safe access to `proxiedRequests`
                dispatchQueue.async {
                    let response = ProxyURLResponse(data: data, response: response, error: error)
                    _ = self.removeRequest(identifier: proxyRequest.id)

                    completionHandler(response)
                }
            }

            // All tasks should have unique identifiers, but if not, cancel the task scheduled
            // earlier.
            let oldTask = self.addRequest(identifier: proxyRequest.id, task: task)
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

extension URLRequestProxy {
    public func sendRequest(_ proxyRequest: ProxyURLRequest) async -> ProxyURLResponse {
        return await withCheckedContinuation { continuation in
            sendRequest(proxyRequest) { proxyResponse in
                continuation.resume(returning: proxyResponse)
            }
        }
    }
}

extension URLRequestProxy: URLRequestProxyProtocol {}
