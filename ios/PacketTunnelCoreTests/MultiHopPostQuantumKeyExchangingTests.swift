//
//  MultiHopPostQuantumKeyExchangingTests.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import WireGuardKitTypes
import XCTest

final class MultiHopPostQuantumKeyExchangingTests: XCTestCase {
    var exitRelay: SelectedRelay!
    var entryRelay: SelectedRelay!

    override func setUpWithError() throws {
        let relayConstraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.country("se")])),
            exitLocations: .only(UserSelectedRelays(locations: [.country("us")]))
        )

        let exitMatch = try RelaySelector.WireGuard.pickCandidate(
            from: try RelaySelector.WireGuard.findCandidates(
                by: relayConstraints.exitLocations,
                in: ServerRelaysResponseStubs.sampleRelays,
                filterConstraint: relayConstraints.filter
            ),
            relays: ServerRelaysResponseStubs.sampleRelays,
            portConstraint: relayConstraints.port,
            numberOfFailedAttempts: 0
        )

        let entryMatch = try RelaySelector.WireGuard.pickCandidate(
            from: try RelaySelector.WireGuard.findCandidates(
                by: relayConstraints.entryLocations,
                in: ServerRelaysResponseStubs.sampleRelays,
                filterConstraint: relayConstraints.filter
            ),
            relays: ServerRelaysResponseStubs.sampleRelays,
            portConstraint: relayConstraints.port,
            numberOfFailedAttempts: 0
        )

        entryRelay = SelectedRelay(
            endpoint: entryMatch.endpoint,
            hostname: entryMatch.relay.hostname,
            location: entryMatch.location
        )
        exitRelay = SelectedRelay(
            endpoint: exitMatch.endpoint,
            hostname: exitMatch.relay.hostname,
            location: exitMatch.location
        )
    }

    func testKeyExchangeFailsWhenNegotiationCannotStart() {
        let expectedNegotiationFailure = expectation(description: "Negotiation failed.")

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 1

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.isInverted = true

        let keyExchangeActor = PostQuantumKeyExchangeActorStub()
        keyExchangeActor.result = .failure(PostQuantumKeyExchangeErrorStub.canceled)

        let multiHopPostQuantumKeyExchanging = MultiHopPostQuantumKeyExchanging(
            entry: entryRelay,
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: keyExchangeActor
        ) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub {
            expectedNegotiationFailure.fulfill()
        }

        multiHopPostQuantumKeyExchanging.start()

        wait(
            for: [expectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testKeyExchangeSuccessWhenNegotiationStart() throws {
        let unexpectedNegotiationFailure = expectation(description: "Negotiation failed.")
        unexpectedNegotiationFailure.isInverted = true

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 3

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = PostQuantumKeyExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let multiHopPostQuantumKeyExchanging = MultiHopPostQuantumKeyExchanging(
            entry: entryRelay,
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: keyExchangeActor
        ) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, ephemeralKey in
            multiHopPostQuantumKeyExchanging.receivePostQuantumKey(preSharedKey, ephemeralKey: ephemeralKey)
        })
        multiHopPostQuantumKeyExchanging.start()

        wait(
            for: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }
}
