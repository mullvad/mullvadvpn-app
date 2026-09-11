// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Testing

fileprivate actor TestCounter {
    var count: Int = 0

    func increment() {
        count += 1
    }
}

struct ActionThrottleTests {
    @Test
    func testEventsAreThrottled() async throws {
        let counter = TestCounter()
        let throttle = ActionThrottle(
            waitInterval: .seconds(5),
            action: { [counter] in
                await counter.increment()
            })
        #expect(await counter.count == 0)
        await throttle.requestAction(force: false)
        try await Task.sleep(nanoseconds: 500_000_000)
        #expect(await counter.count == 1)
        await throttle.requestAction(force: false)
        try await Task.sleep(nanoseconds: 500_000_000)
        #expect(await counter.count == 1)
    }

    @Test func testForcedActionsAreExecuted() async throws {
        let counter = TestCounter()
        let throttle = ActionThrottle(
            waitInterval: .seconds(5),
            action: { [counter] in
                await counter.increment()
            })
        #expect(await counter.count == 0)
        await throttle.requestAction(force: false)
        try await Task.sleep(nanoseconds: 500_000_000)
        #expect(await counter.count == 1)
        await throttle.requestAction(force: true)
        try await Task.sleep(nanoseconds: 500_000_000)
        #expect(await counter.count == 2)
    }

    @Test func testForcedActionsSetThrottleTimer() async throws {
        let counter = TestCounter()
        let throttle = ActionThrottle(
            waitInterval: .seconds(5),
            action: { [counter] in
                await counter.increment()
            })
        #expect(await counter.count == 0)
        await throttle.requestAction(force: true)
        try await Task.sleep(nanoseconds: 500_000_000)
        #expect(await counter.count == 1)
        await throttle.requestAction(force: false)
        try await Task.sleep(nanoseconds: 500_000_000)
        #expect(await counter.count == 1)
    }
}
