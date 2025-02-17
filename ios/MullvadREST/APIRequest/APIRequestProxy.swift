//
//  APIRequestProxy.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadTypes

public protocol APIRequestProxyProtocol {
    func sendRequest(_ proxyRequest: ProxyAPIRequest, completion: @escaping @Sendable (ProxyAPIResponse) -> Void)
    func sendRequest(_ proxyRequest: ProxyAPIRequest) async -> ProxyAPIResponse
    func cancelRequest(identifier: UUID)
}

/// Network request proxy capable of passing serializable requests and responses over the given transport provider.
public final class APIRequestProxy: APIRequestProxyProtocol, @unchecked Sendable {
    /// Serial queue used for synchronizing access to class members.
    private let dispatchQueue: DispatchQueue

    private let transportProvider: APITransportProviderProtocol

    /// List of all proxied network requests bypassing VPN.
    private var proxiedRequests: [UUID: Cancellable] = [:]

    public init(
        dispatchQueue: DispatchQueue,
        transportProvider: APITransportProviderProtocol
    ) {
        self.dispatchQueue = dispatchQueue
        self.transportProvider = transportProvider
    }

    public func sendRequest(
        _ proxyRequest: ProxyAPIRequest,
        completion: @escaping @Sendable (ProxyAPIResponse) -> Void
    ) {
        dispatchQueue.async {
            guard let transport = self.transportProvider.makeTransport() else {
                completion(ProxyAPIResponse(data: nil, error: nil))
                return
            }

            let cancellable = transport.sendRequest(proxyRequest.request) { [weak self] response in
                guard let self else { return }

                // Use `dispatchQueue` to guarantee thread safe access to `proxiedRequests`
                dispatchQueue.async {
                    _ = self.removeRequest(identifier: proxyRequest.id)
                    completion(response)
                }
            }

            // All tasks should have unique identifiers, but if not, cancel the task scheduled earlier.
            let oldTask = self.addRequest(identifier: proxyRequest.id, task: cancellable)
            oldTask?.cancel()
        }
    }

    public func sendRequest(_ proxyRequest: ProxyAPIRequest) async -> ProxyAPIResponse {
        return await withCheckedContinuation { continuation in
            sendRequest(proxyRequest) { proxyResponse in
                continuation.resume(returning: proxyResponse)
            }
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
