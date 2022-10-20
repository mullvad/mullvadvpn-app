//
//  RESTTransport.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-03.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public protocol RESTTransport: AnyObject {
    var name: String { get }

    func sendRequest(
        _ request: URLRequest,
        completion: @escaping (Data?, URLResponse?, Error?) -> Void
    ) throws -> Cancellable
}
