//
//  URLSessionProtocol.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-16.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol URLSessionProtocol {
    func data(for request: URLRequest) async throws -> (Data, URLResponse)
}

extension URLSession: URLSessionProtocol {}
