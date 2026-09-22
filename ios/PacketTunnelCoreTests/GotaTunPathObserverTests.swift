// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Network
import XCTest
import os

@testable import PacketTunnelCore

private typealias Path = GotaTunPathObserver.Path

final class GotaTunPathObserverTests: XCTestCase {
    private let wifi = Path(status: .satisfied, interface: "en0")
    private let cellular = Path(status: .satisfied, interface: "pdp_ip0")
    private let lost = Path(status: .unsatisfied, interface: nil)
    private let lostOnCellular = Path(status: .unsatisfied, interface: "pdp_ip0")

    private let debounce = GotaTunPathObserver.pathUpdateDebounceDelay
    private let addressCheck = GotaTunPathObserver.addressCheckDelay

    private var clock: TestClock!
    private var addresses: AddressBook!
    private var deliveries: Deliveries!
    private var paths: AsyncStream<Path>.Continuation!
    private var observer: GotaTunPathObserver!

    override func setUp() async throws {
        clock = TestClock()
        addresses = AddressBook()
        deliveries = Deliveries()
        addresses.set(["10.0.0.2"], for: wifi.interface)
    }

    override func tearDown() async throws {
        await observer?.stop()
        paths?.finish()
    }

    @discardableResult
    private func start(with initial: Path) async -> NWPath.Status {
        let (stream, continuation) = AsyncStream<Path>.makeStream()
        paths = continuation
        continuation.yield(initial)

        let addresses = addresses!
        let deliveries = deliveries!
        observer = GotaTunPathObserver(
            clock: clock,
            pathUpdates: { stream },
            localAddresses: { addresses.get($0) }
        )
        return await observer.start { deliveries.record($0) }
    }

    // MARK: - Start

    func testStartReturnsInitialStatusWithoutDelivering() async {
        let status = await start(with: wifi)

        XCTAssertEqual(status, .satisfied)
        XCTAssertEqual(deliveries.all, [])
        XCTAssertEqual(clock.sleeperCount, 0)
    }

    func testStartTwiceReturnsFirstStatus() async {
        await start(with: lost)
        let status = await observer.start { _ in XCTFail("Second body must not be installed") }

        XCTAssertEqual(status, .unsatisfied)
    }

    func testUpdatesFromPathSourceAreDelivered() async {
        await start(with: wifi)

        paths.yield(cellular)

        await deliveries.wait(for: 1)
        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    // MARK: - Path changes

    func testInterfaceChangeIsDeliveredWithoutDelay() async {
        await start(with: wifi)

        await observer.handle(cellular)

        XCTAssertEqual(deliveries.all, [.satisfied])
        XCTAssertEqual(clock.sleeperCount, 0)
    }

    func testGatewayChangeIsDelivered() async {
        await start(with: wifi)

        await observer.handle(
            Path(status: .satisfied, interface: "en0", gateways: [.hostPort(host: "10.0.0.1", port: 0)])
        )

        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    func testRecoveryFromUnsatisfiedIsDeliveredWithoutDelay() async {
        await start(with: lost)

        await observer.handle(wifi)

        XCTAssertEqual(deliveries.all, [.satisfied])
        XCTAssertEqual(clock.sleeperCount, 0)
    }

    // MARK: - Loss debounce

    func testLossIsDeliveredOnlyAfterDebounce() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await clock.advance(by: debounce - .milliseconds(1))

        XCTAssertEqual(deliveries.all, [])
        XCTAssertEqual(clock.sleeperCount, 1)

        await clock.advance(by: .milliseconds(1))

        await deliveries.wait(for: 1)
        XCTAssertEqual(deliveries.all, [.unsatisfied])
        XCTAssertEqual(clock.sleeperCount, 0)
    }

    func testTransientLossIsSwallowedWhenSamePathReturns() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await clock.advance(by: debounce / 2)
        await observer.handle(wifi)
        await clock.advance(by: debounce)

        XCTAssertEqual(deliveries.all, [])
    }

    func testLossIsCancelledByNewPath() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await observer.handle(cellular)
        await clock.runToCompletion()

        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    func testRepeatedLossDoesNotPostponeDelivery() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await clock.advance(by: debounce - .milliseconds(1))
        await observer.handle(lost)

        XCTAssertEqual(clock.sleeperCount, 1)
        XCTAssertEqual(deliveries.all, [])

        await clock.advance(by: .milliseconds(1))

        await deliveries.wait(for: 1)
        XCTAssertEqual(deliveries.all, [.unsatisfied])
        XCTAssertEqual(clock.sleeperCount, 0)
    }

    func testLossChurnIsDeliveredOnce() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        for _ in 0..<9 {
            await clock.advance(by: debounce / 5)
            await observer.handle(lostOnCellular)
            await observer.handle(lost)
        }

        XCTAssertEqual(deliveries.all, [.unsatisfied])
    }

    func testLatestLossPathIsDelivered() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await observer.handle(lostOnCellular)
        await clock.advance(by: debounce)
        await deliveries.wait(for: 1)

        await observer.handle(lostOnCellular)

        XCTAssertEqual(clock.sleeperCount, 0)
        XCTAssertEqual(deliveries.all, [.unsatisfied])
    }

    func testLossChurnIsSwallowedWhileSamePathKeepsReturning() async {
        await start(with: wifi)

        for _ in 0..<10 {
            await observer.handle(lost)
            await clock.advance(by: debounce / 2)
            await observer.handle(wifi)
        }
        await clock.advance(by: debounce)

        XCTAssertEqual(deliveries.all, [])
    }

    func testJoiningWifiDuringLossIsDeliveredAsSatisfied() async {
        await start(with: cellular)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await observer.handle(wifi)

        XCTAssertEqual(deliveries.all, [.satisfied])
        await clock.advance(by: debounce)
        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    func testDeliveredLossIsNotRepeated() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await clock.advance(by: debounce)
        await deliveries.wait(for: 1)

        await observer.handle(lost)
        await clock.runToCompletion()

        XCTAssertEqual(deliveries.all, [.unsatisfied])
    }

    // MARK: - Address check

    func testAddressChangeIsDeliveredAfterCheckDelay() async {
        await start(with: wifi)
        addresses.set(["10.0.0.3"], for: wifi.interface)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck - .milliseconds(1))

        XCTAssertEqual(deliveries.all, [])

        await clock.advance(by: .milliseconds(1))

        await deliveries.wait(for: 1)
        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    func testUnchangedAddressesAreNotDelivered() async {
        await start(with: wifi)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck)

        XCTAssertEqual(clock.sleeperCount, 0)
        XCTAssertEqual(deliveries.all, [])
    }

    func testAddressCheckIsNotPostponedByFurtherUpdates() async {
        await start(with: wifi)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck - .seconds(1))
        addresses.set(["10.0.0.3"], for: wifi.interface)
        await observer.handle(wifi)

        XCTAssertEqual(clock.sleeperCount, 1)

        await clock.advance(by: .seconds(1))

        await deliveries.wait(for: 1)
        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    func testAddressCheckIsRescheduledAfterCompleting() async {
        await start(with: wifi)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck)
        XCTAssertEqual(clock.sleeperCount, 0)

        addresses.set(["10.0.0.3"], for: wifi.interface)
        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck)

        await deliveries.wait(for: 1)
        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    func testDeliveredAddressesBecomeTheNewBaseline() async {
        await start(with: wifi)
        addresses.set(["10.0.0.3"], for: wifi.interface)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck)
        await deliveries.wait(for: 1)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck)

        XCTAssertEqual(deliveries.all, [.satisfied])
    }

    func testAddressCheckIsNotScheduledWhileUnsatisfied() async {
        await start(with: lost)

        await observer.handle(lost)

        XCTAssertEqual(clock.sleeperCount, 0)
    }

    func testAddressCheckIsSkippedWhileLossIsPending() async {
        await start(with: wifi)
        addresses.set(["10.0.0.3"], for: wifi.interface)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await clock.advance(by: addressCheck - debounce / 2)
        await observer.handle(lost)
        await clock.waitForSleepers(2)
        await clock.advance(by: debounce / 2)

        XCTAssertEqual(deliveries.all, [])
        XCTAssertEqual(clock.sleeperCount, 1)

        await clock.advance(by: debounce / 2)

        await deliveries.wait(for: 1)
        XCTAssertEqual(deliveries.all, [.unsatisfied])
    }

    func testAddressCheckIsSkippedAfterLossIsDelivered() async {
        await start(with: wifi)
        addresses.set(["10.0.0.3"], for: wifi.interface)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await observer.handle(lost)
        await clock.waitForSleepers(2)
        await clock.runToCompletion()

        XCTAssertEqual(deliveries.all, [.unsatisfied])
    }

    // MARK: - Stop

    func testStopCancelsPendingLoss() async {
        await start(with: wifi)

        await observer.handle(lost)
        await clock.waitForSleepers(1)
        await observer.stop()

        XCTAssertEqual(clock.sleeperCount, 0)
        XCTAssertEqual(deliveries.all, [])
    }

    func testStopCancelsPendingAddressCheck() async {
        await start(with: wifi)
        addresses.set(["10.0.0.3"], for: wifi.interface)

        await observer.handle(wifi)
        await clock.waitForSleepers(1)
        await observer.stop()

        XCTAssertEqual(clock.sleeperCount, 0)
        XCTAssertEqual(deliveries.all, [])
    }
}

/// Interface addresses the observer under test sees.
private final class AddressBook: Sendable {
    private let addresses = OSAllocatedUnfairLock<[String: Set<String>]>(initialState: [:])

    func set(_ value: Set<String>, for interface: String?) {
        guard let interface else { return }
        addresses.withLock { $0[interface] = value }
    }

    func get(_ interface: String?) -> Set<String> {
        guard let interface else { return [] }
        return addresses.withLock { $0[interface] ?? [] }
    }
}

/// Statuses delivered by the observer under test, in order.
private final class Deliveries: Sendable {
    private let statuses = OSAllocatedUnfairLock<[NWPath.Status]>(initialState: [])

    var all: [NWPath.Status] {
        statuses.withLock { $0 }
    }

    func record(_ status: NWPath.Status) {
        statuses.withLock { $0.append(status) }
    }

    /// Wait until at least `count` statuses have been delivered.
    func wait(for count: Int, file: StaticString = #filePath, line: UInt = #line) async {
        for _ in 0..<10_000 {
            if all.count >= count { return }
            await Task.yield()
        }
        XCTFail("Timed out waiting for \(count) deliveries, found \(all.count)", file: file, line: line)
    }
}
