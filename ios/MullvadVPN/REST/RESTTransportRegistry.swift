//
//  RESTTransportRegistry.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-06.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

class RESTTransportRegistry {
    static let shared = RESTTransportRegistry()

    private var transport: RESTTransport?
    private let nslock = NSLock()

    private init() {}

    func setTransport(_ transport: RESTTransport) {
        nslock.lock()
        defer { nslock.unlock() }

        self.transport = transport
    }

    func getTransport() -> RESTTransport? {
        nslock.lock()
        defer { nslock.unlock() }

        return transport
    }
}
