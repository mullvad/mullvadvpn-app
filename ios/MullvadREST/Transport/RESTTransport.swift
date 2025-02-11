//
//  RESTTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes

public protocol RESTTransport: Sendable {
    var name: String { get }

    func sendRequest(_ request: URLRequest, completion: @escaping @Sendable (Data?, URLResponse?, Error?) -> Void)
        -> Cancellable
}

public protocol APITransport: Sendable {
    var name: String { get }

    func sendRequest(_ request: MullvadApiRequest, completion: @escaping @Sendable (MullvadApiResponse) -> Void)
    -> Cancellable
}
