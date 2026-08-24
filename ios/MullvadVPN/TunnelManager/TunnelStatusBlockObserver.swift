// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import NetworkExtension

final class TunnelStatusBlockObserver: TunnelStatusObserver, @unchecked Sendable {
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
        let block: @Sendable () -> Void = {
            self.handler(tunnel, status)
        }

        if let queue {
            queue.async(execute: block)
        } else {
            block()
        }
    }
}
