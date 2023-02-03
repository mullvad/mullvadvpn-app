//
//  URLRequestProxy.swift
//  PacketTunnel
//
//  Created by pronebird on 03/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public final class URLRequestProxy {
    /// Serial queue used for synchronizing access to class members.
    private let dispatchQueue: DispatchQueue

    /// URL session used for proxy requests.
    private let urlSession: URLSession

    /// List of all proxied network requests bypassing VPN.
    private var proxiedRequests: [UUID: URLSessionDataTask] = [:]

    public init(urlSession: URLSession, dispatchQueue: DispatchQueue) {
        self.urlSession = urlSession
        self.dispatchQueue = dispatchQueue
    }

    public func sendRequest(
        _ proxyRequest: ProxyURLRequest,
        completionHandler: @escaping (ProxyURLResponse) -> Void
    ) {
        dispatchQueue.async {
            let task = self.urlSession
                .dataTask(with: proxyRequest.urlRequest) { [weak self] data, response, error in
                    guard let self = self else { return }

                    self.dispatchQueue.async {
                        let response = ProxyURLResponse(
                            data: data,
                            response: response,
                            error: error
                        )

                        _ = self.removeRequest(identifier: proxyRequest.id)

                        completionHandler(response)
                    }
                }

            // All tasks should have unique identifiers, but if not, cancel the task scheduled
            // earlier.
            let oldTask = self.addRequest(identifier: proxyRequest.id, task: task)
            oldTask?.cancel()

            task.resume()
        }
    }

    public func cancelRequest(identifier: UUID) {
        dispatchQueue.async {
            let task = self.removeRequest(identifier: identifier)

            task?.cancel()
        }
    }

    private func addRequest(identifier: UUID, task: URLSessionDataTask) -> URLSessionDataTask? {
        return proxiedRequests.updateValue(task, forKey: identifier)
    }

    private func removeRequest(identifier: UUID) -> URLSessionDataTask? {
        return proxiedRequests.removeValue(forKey: identifier)
    }
}
