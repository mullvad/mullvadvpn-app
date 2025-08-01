//
//  URLSessionTransport.swift
//  MullvadTransport
//
//  Created by Mojgan on 2023-12-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct URLSessionTaskWrapper: Cancellable {
    let task: URLSessionTask
    func cancel() {
        task.cancel()
    }
}

public final class URLSessionTransport: RESTTransport {
    public var name: String {
        "url-session"
    }

    public let urlSession: URLSession

    public init(urlSession: URLSession) {
        self.urlSession = urlSession
    }

    public func sendRequest(
        _ request: URLRequest,
        completion: @escaping @Sendable (Data?, URLResponse?, Swift.Error?) -> Void
    ) -> Cancellable {
        let dataTask = urlSession.dataTask(with: request, completionHandler: completion)
        dataTask.resume()
        return URLSessionTaskWrapper(task: dataTask)
    }
}
