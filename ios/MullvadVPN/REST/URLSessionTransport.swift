//
//  URLSessionTransport.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension URLSessionTask: Cancellable {}

final class URLSessionTransport: RESTTransport {
    var name: String {
        return "url-session"
    }

    let urlSession: URLSession

    init(urlSession: URLSession) {
        self.urlSession = urlSession
    }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) throws -> Cancellable {
        let dataTask = urlSession.dataTask(with: request, completionHandler: completion)
        dataTask.resume()
        return dataTask
    }
}
