//
//  SingleResumerTests.swift
//  MullvadVPNTests
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

final class SingleResumerTests: XCTestCase {
    func testResumesWhenResultArrivesAfterContinuationSet() async throws {
        let resumer = SingleResumer<Int>()

        let value = try await withCheckedThrowingContinuation { continuation in
            resumer.setContinuation(continuation)
            resumer.resume(with: .success(1))
        }

        XCTAssertEqual(value, 1)
    }

    func testResumesWhenResultArrivesBeforeContinuationSet() async throws {
        let resumer = SingleResumer<Int>()
        resumer.resume(with: .success(2))

        let value = try await withCheckedThrowingContinuation { continuation in
            resumer.setContinuation(continuation)
        }

        XCTAssertEqual(value, 2)
    }

    func testFirstResultWins() async throws {
        // A second resume of a checked continuation traps, so completing this test at all
        // asserts single-resumption; the value asserts first-wins ordering.
        let resumer = SingleResumer<Int>()

        let value = try await withCheckedThrowingContinuation { continuation in
            resumer.setContinuation(continuation)
            resumer.resume(with: .success(1))
            resumer.resume(with: .success(99))
            resumer.resume(with: .failure(CancellationError()))
        }

        XCTAssertEqual(value, 1)
    }

    func testFirstResultWinsBeforeContinuationSet() async throws {
        let resumer = SingleResumer<Int>()
        resumer.resume(with: .success(1))
        resumer.resume(with: .success(99))

        let value = try await withCheckedThrowingContinuation { continuation in
            resumer.setContinuation(continuation)
        }

        XCTAssertEqual(value, 1)
    }

    func testPropagatesFailure() async {
        let resumer = SingleResumer<Int>()

        do {
            _ = try await withCheckedThrowingContinuation { continuation in
                resumer.setContinuation(continuation)
                resumer.resume(with: .failure(CancellationError()))
            }
            XCTFail("Expected CancellationError")
        } catch {
            XCTAssertTrue(error is CancellationError)
        }
    }

    func testConcurrentRacersResumeExactlyOnce() async throws {
        // Hammer the racy path: multiple concurrent resume calls racing each other and
        // racing setContinuation. A double-resume traps, an unresumed continuation hangs
        // the test — so completion with any racer's value is the assertion.
        for _ in 0..<500 {
            let resumer = SingleResumer<Int>()

            let value = try await withThrowingTaskGroup(of: Int?.self) { group in
                group.addTask {
                    try await withCheckedThrowingContinuation { continuation in
                        resumer.setContinuation(continuation)
                    }
                }

                for racerIndex in 0..<4 {
                    group.addTask {
                        resumer.resume(with: .success(racerIndex))
                        return nil
                    }
                }

                var winner: Int?
                for try await result in group {
                    if let result {
                        winner = result
                    }
                }
                return winner
            }

            XCTAssertNotNil(value)
            XCTAssertTrue((0..<4).contains(value!))
        }
    }
}
