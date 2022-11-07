//
//  URLSessionTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension URLSessionTask: Cancellable {}

extension REST {
    public final class URLSessionTransport: RESTTransport {
        public var name: String {
            return "url-session"
        }

        public let urlSession: URLSession

        public init(urlSession: URLSession) {
            self.urlSession = urlSession
        }

        public func sendRequest(
            _ request: URLRequest,
            completion: @escaping (Data?, URLResponse?, Swift.Error?) -> Void
        ) throws -> Cancellable {
            let dataTask = urlSession.dataTask(with: request, completionHandler: completion)
            dataTask.resume()
            return dataTask
        }
    }
}
