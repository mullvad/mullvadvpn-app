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

    private var timeoutCount = 0
    private var transports: [RESTTransport] = []

    private let nslock = NSLock()

    private init() {}

    func setTransports(_ transports: [RESTTransport]) {
        nslock.lock()
        defer { nslock.unlock() }

        self.transports = transports
        timeoutCount = 0
    }

    func transportDidFinishLoad(_ transport: RESTTransport) {
        nslock.lock()
        defer { nslock.unlock() }

        guard let firstTransport = getTransport(),
              firstTransport === transport
        else { return }

        timeoutCount = 0
    }

    func transportDidTimeout(_ transport: RESTTransport) {
        nslock.lock()
        defer { nslock.unlock() }

        guard let firstTransport = getTransport(),
              firstTransport === transport
        else { return }

        timeoutCount += 1

        if timeoutCount > 5 {
            transports.removeFirst() // remove current transport
            transports.append(transport) // take current transport and put it in the back

            timeoutCount = 0
        }
    }

    func getTransport() -> RESTTransport? {
        nslock.lock()
        defer { nslock.unlock() }

        return transports.first
    }
}
