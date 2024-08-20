//
//  SingleHopEphemeralPeerExchangerTests.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import WireGuardKitTypes
import XCTest

final class SingleHopEphemeralPeerExchangerTests: XCTestCase {
    var exitRelay: SelectedRelay!

    override func setUpWithError() throws {
        let relayConstraints = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.hostname("se", "sto", "se6-wireguard")]))
        )

        let candidates = try RelaySelector.WireGuard.findCandidates(
            by: relayConstraints.exitLocations,
            in: ServerRelaysResponseStubs.sampleRelays,
            filterConstraint: relayConstraints.filter,
            daitaEnabled: false
        )

        let match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            relays: ServerRelaysResponseStubs.sampleRelays,
            portConstraint: relayConstraints.port,
            numberOfFailedAttempts: 0
        )

        exitRelay = SelectedRelay(endpoint: match.endpoint, hostname: match.relay.hostname, location: match.location)
    }

    func testKeyExchangeFailsWhenNegotiationCannotStart() {
        let expectedNegotiationFailure = expectation(description: "Negotiation failed.")

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 1

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.isInverted = true

        let keyExchangeActor = EphemeralPeerExchangeActorStub()
        keyExchangeActor.result = .failure(EphemeralPeerExchangeErrorStub.canceled)

        let singleHopPostQuantumKeyExchanging = SingleHopEphemeralPeerExchanger(
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: keyExchangeActor,
            enablePostQuantum: true,
            enableDaita: false
        ) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub {
            expectedNegotiationFailure.fulfill()
        }

        singleHopPostQuantumKeyExchanging.start()

        wait(
            for: [expectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testKeyExchangeSuccessWhenNegotiationStart() throws {
        let unexpectedNegotiationFailure = expectation(description: "Negotiation failed.")
        unexpectedNegotiationFailure.isInverted = true

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 2

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let singleHopPostQuantumKeyExchanging = SingleHopEphemeralPeerExchanger(
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: keyExchangeActor,
            enablePostQuantum: true,
            enableDaita: false
        ) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, ephemeralKey in
            singleHopPostQuantumKeyExchanging.receivePostQuantumKey(preSharedKey, ephemeralKey: ephemeralKey)
        })
        singleHopPostQuantumKeyExchanging.start()

        wait(
            for: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }
}
