//
//  RESTTransportRegistry.swift
//  MullvadREST
//
//  Created by Sajad Vishkai on 2022-10-06.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public final class TransportRegistry {
        public static let shared = TransportRegistry()

        private var transport: RESTTransport?
        private let nslock = NSLock()

        private init() {}

        public func setTransport(_ transport: RESTTransport) {
            nslock.lock()
            defer { nslock.unlock() }

            self.transport = transport
        }

        public func getTransport() -> RESTTransport? {
            nslock.lock()
            defer { nslock.unlock() }

            return transport
        }
    }
}
