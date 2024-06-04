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
    private var multihopStateListener: MultihopStateListener!
    private var multihopUpdater: MultihopUpdater!
    private var observers: [MultihopObserver]!

    override func setUp() {
        multihopStateListener = MultihopStateListener()
        multihopUpdater = MultihopUpdater(listener: multihopStateListener)
        observers = []
    }

    override func tearDown() {
        self.observers.forEach {
            multihopUpdater.removeObserver($0)
        }
    }

    func testMultipleListener() {
        var count = 0

        observers.append(MultihopObserverBlock(didUpdateMultihop: { _, _ in
            count += 1
        }))

        observers.append(MultihopObserverBlock(didUpdateMultihop: { _, _ in
            count += 1
        }))

        observers.forEach { multihopUpdater.addObserver($0) }

        multihopStateListener.onNewMultihop?(.on)

        XCTAssertEqual(count, 2)
    }
}
