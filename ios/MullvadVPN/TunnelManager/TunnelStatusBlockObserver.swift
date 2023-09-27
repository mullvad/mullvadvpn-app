//
//  TunnelStatusBlockObserver.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

final class TunnelStatusBlockObserver: TunnelStatusObserver {
    typealias Handler = (any TunnelProtocol, NEVPNStatus) -> Void

    private weak var tunnel: (any TunnelProtocol)?
    private let queue: DispatchQueue?
    private let handler: Handler

    init(tunnel: any TunnelProtocol, queue: DispatchQueue?, handler: @escaping Handler) {
        self.tunnel = tunnel
        self.queue = queue
        self.handler = handler
    }

    func invalidate() {
        tunnel?.removeObserver(self)
    }

    func tunnel(_ tunnel: any TunnelProtocol, didReceiveStatus status: NEVPNStatus) {
        let block = {
            self.handler(tunnel, status)
        }

        if let queue {
            queue.async(execute: block)
        } else {
            block()
        }
    }
}
