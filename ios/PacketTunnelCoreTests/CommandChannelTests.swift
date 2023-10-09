//
//  CommandChannelTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 27/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import PacketTunnelCore
import XCTest

final class CommandChannelTests: XCTestCase {
    func testCoalescingReconnect() async {
        let channel = CommandChannel()

        channel.send(.start(StartOptions(launchSource: .app)))
        channel.send(.reconnect(.random))
        channel.send(.reconnect(.random))
        channel.send(.switchKey)
        channel.send(.reconnect(.current))
        channel.sendEnd()

        let commands = await channel.map { $0.primitiveCommand }.collect()

        XCTAssertEqual(commands, [.start, .switchKey, .reconnect(.current)])
    }

    /// Test that stops cancels all preceding tasks.
    func testCoalescingStop() async {
        let channel = CommandChannel()

        channel.send(.start(StartOptions(launchSource: .app)))
        channel.send(.reconnect(.random))
        channel.send(.stop)
        channel.send(.reconnect(.current))
        channel.send(.stop)
        channel.send(.switchKey)
        channel.sendEnd()

        let commands = await channel.map { $0.primitiveCommand }.collect()

        XCTAssertEqual(commands, [.stop, .switchKey])
    }

    /// Test that iterations over the finished channel yield `nil`.
    func testFinishFlushingUnconsumedValues() async {
        let channel = CommandChannel()
        channel.send(.stop)
        channel.finish()

        let value = await channel.makeAsyncIterator().next()
        XCTAssertNil(value)
    }

    /// Test that the call to `finish()` ends the iteration that began prior to that.
    func testFinishEndsAsyncIterator() async throws {
        let channel = CommandChannel()
        let expectFinish = expectation(description: "Call to finish()")
        let expectEndIteration = expectation(description: "Iteration over channel should end upon call to finish()")

        // Start iterating over commands in channel. The for-await loop should suspend the continuation.
        Task {
            for await command in channel {
                print(command)
            }

            expectEndIteration.fulfill()
        }

        // Tell channel to finish() after a small delay. This should resume execution in the task above and exit the
        // for-await loop.
        Task {
            try await Task.sleep(nanoseconds: 1_000_000)

            expectFinish.fulfill()
            channel.finish()
        }

        await fulfillment(of: [expectFinish, expectEndIteration], timeout: 100, enforceOrder: true)
    }
}

extension AsyncSequence {
    func collect() async rethrows -> [Element] {
        try await reduce(into: [Element]()) { $0.append($1) }
    }
}

/// Primitive version of `Command` that can be used in tests and easily compared against.
enum PrimitiveCommand: Equatable {
    case start, stop, reconnect(NextRelay), switchKey, other
}

extension Command {
    var primitiveCommand: PrimitiveCommand {
        switch self {
        case .start:
            return .start
        case let .reconnect(nextRelay, _):
            return .reconnect(nextRelay)
        case .switchKey:
            return .switchKey
        case .stop:
            return .stop
        default:
            return .other
        }
    }
}
