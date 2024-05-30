//
//  MultihopUpdaterTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-05-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import XCTest

class MultihopUpdaterTests: XCTestCase {
    func testMultipleListener() {
        let multihopStateListener = MultihopStateListener()
        let multihopPropagator = MultihopUpdater(listener: multihopStateListener)

        var count = 0

        multihopPropagator.addObserver(MultihopObserverBlock(didUpdateMultihop: { _, _ in
            count += 1
        }))

        multihopPropagator.addObserver(MultihopObserverBlock(didUpdateMultihop: { _, _ in
            count += 1
        }))

        multihopStateListener.onNewMultihop?(.on)

        XCTAssertEqual(count, 2)
    }
}
