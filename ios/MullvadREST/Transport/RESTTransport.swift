//
//  RESTTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol RESTTransport: Sendable {
    var name: String { get }

    func sendRequest(_ request: URLRequest, completion: @escaping @Sendable (Data?, URLResponse?, Error?) -> Void)
        -> Cancellable
}
