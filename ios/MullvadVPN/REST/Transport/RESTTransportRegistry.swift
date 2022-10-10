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

    private init() {}

    func register(_ transport: RESTTransport) {
        if !contains(transport) {
            transports.append(transport)
        }
    }

    func unregister(_ transport: RESTTransport) {
        guard let index = firstIndex(transport) else { return }
        transports.remove(at: index)
    }

    func findElement(_ element: RESTTransport) -> (index: Int, element: RESTTransport)? {
        for i in 0 ..< transports.count {
            if transports[i] == element {
                return (i, transports[i])
            }
        }

        return nil
    }

    func contains(_ element: RESTTransport) -> Bool {
        findElement(element) != nil
    }

    func firstIndex(_ element: RESTTransport) -> Int? {
        findElement(element)?.index
    }
}

extension RESTTransportRegistry {
    func transportDidFinishLoad(_ transport: RESTTransport) {
        guard let firstTransport = transports.first,
              firstTransport == transport
        else { return }

        timeoutCount = 0
    }

    func transportDidTimeout(_ transport: RESTTransport) {
        guard let firstTransport = transports.first,
              firstTransport == transport
        else { return }

        timeoutCount += 1

        if timeoutCount > 5 {
            transports.removeFirst() // remove current transport
            transports.append(transport) // take current transport and put it in the back
            timeoutCount = 0
        }
    }

    func getTransport() -> RESTTransport? {
        return transports.first
    }
}
