//
//  PacketTunnelActorStub.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore
import XCTest

struct PacketTunnelActorStub: PacketTunnelActorProtocol {
    let innerState: ObservedState = .disconnected
    var stateExpectation: XCTestExpectation?
    var reconnectExpectation: XCTestExpectation?
    var keyRotationExpectation: XCTestExpectation?

    var observedState: ObservedState {
        get async {
            stateExpectation?.fulfill()
            return innerState
        }
    }

    func reconnect(to nextRelay: NextRelay) {
        reconnectExpectation?.fulfill()
    }

    func notifyKeyRotation(date: Date?) {
        keyRotationExpectation?.fulfill()
    }
}
