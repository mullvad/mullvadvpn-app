//
//  URLRequestProxyProtocol.swift
//  PacketTunnelCore
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol URLRequestProxyProtocol {
    func sendRequest(_ proxyRequest: ProxyURLRequest, completionHandler: @escaping @Sendable (ProxyURLResponse) -> Void)
    func sendRequest(_ proxyRequest: ProxyURLRequest) async -> ProxyURLResponse
    func cancelRequest(identifier: UUID)
}
