//
//  URLSessionProtocol.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public protocol URLSessionProtocol {
    func data(for request: URLRequest) async throws -> (Data, URLResponse)
}

extension URLSession: URLSessionProtocol {}
