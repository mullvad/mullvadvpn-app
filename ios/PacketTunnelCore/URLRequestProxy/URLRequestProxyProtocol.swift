//
//  URLRequestProxyProtocol.swift
//  PacketTunnelCore
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

public protocol URLRequestProxyProtocol {
    func sendRequest(_ proxyRequest: ProxyURLRequest, completionHandler: @escaping @Sendable (ProxyURLResponse) -> Void)
    func sendRequest(_ proxyRequest: ProxyURLRequest) async -> ProxyURLResponse
    func cancelRequest(identifier: UUID)
}

public protocol APIRequestProxyProtocol {
    func sendRequest(_ proxyRequest: ProxyAPIRequest, completion: @escaping @Sendable (ProxyAPIResponse) -> Void)
    func sendRequest(_ proxyRequest: ProxyAPIRequest) async -> ProxyAPIResponse
    func cancelRequest(identifier: UUID)
}
