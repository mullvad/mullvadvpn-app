//
//  URLSessionTransport.swift
//  MullvadREST
//
//  Created by Mojgan on 2023-12-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension URLSessionTask: Cancellable {}

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
        completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
    ) -> Cancellable {
        let dataTask = urlSession.dataTask(with: request, completionHandler: completion)
        dataTask.resume()
        return dataTask
    }
}
