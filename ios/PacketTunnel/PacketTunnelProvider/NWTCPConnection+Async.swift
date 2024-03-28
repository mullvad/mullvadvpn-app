//
//  NWTCPConnection+Async.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-02-27.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

class KVOWrapper {
    let observation: NSKeyValueObservation

    init(observation: NSKeyValueObservation) {
        self.observation = observation
    }

    deinit {
        NSLog("Bye cruel life")
    }
}

extension NWTCPConnection {
    var viability: AsyncStream<Bool> {
        AsyncStream { continuation in
            let keyPath: KeyPath<NWTCPConnection, Bool> = \.isViable

            let isViableObserver = observe(keyPath, options: [.new]) { connection, _ in
                continuation.yield(connection.isViable)
            }

            let wrapper = KVOWrapper(observation: isViableObserver)

            continuation.onTermination = { @Sendable _ in
                wrapper.observation.invalidate()
            }

            continuation.yield(false)
        }
    }
}
