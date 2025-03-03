//
//  MultiHopEphemeralPeerExchangerTests.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import WireGuardKitTypes
import XCTest

final class MultiHopEphemeralPeerExchangerTests: XCTestCase {
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
                filterConstraint: relayConstraints.filter,
                daitaEnabled: false
            ),
            wireguard: ServerRelaysResponseStubs.sampleRelays.wireguard,
            portConstraint: relayConstraints.port,
            numberOfFailedAttempts: 0
        )

        let entryMatch = try RelaySelector.WireGuard.pickCandidate(
            from: try RelaySelector.WireGuard.findCandidates(
                by: relayConstraints.entryLocations,
                in: ServerRelaysResponseStubs.sampleRelays,
                filterConstraint: relayConstraints.filter,
                daitaEnabled: false
            ),
            wireguard: ServerRelaysResponseStubs.sampleRelays.wireguard,
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

    func testEphemeralPeerExchangeFailsWhenNegotiationCannotStart() async {
        let expectedNegotiationFailure = expectation(description: "Negotiation failed.")

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 1

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.isInverted = true

        let peerExchangeActor = EphemeralPeerExchangeActorStub()
        peerExchangeActor.result = .failure(EphemeralPeerExchangeErrorStub.canceled)

        let multiHopExchanger = MultiHopEphemeralPeerExchanger(
            entry: entryRelay,
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: peerExchangeActor,
            enablePostQuantum: true,
            enableDaita: false
        ) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        peerExchangeActor.delegate = KeyExchangingResultStub {
            expectedNegotiationFailure.fulfill()
        }

        await multiHopExchanger.start()

        wait(
            for: [expectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testEphemeralPeerExchangeSuccessWhenPostQuantumNegotiationStarts() async throws {
        let unexpectedNegotiationFailure = expectation(description: "Negotiation failed.")
        unexpectedNegotiationFailure.isInverted = true

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 3

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let peerExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        peerExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let multiHopPeerExchanger = MultiHopEphemeralPeerExchanger(
            entry: entryRelay,
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: peerExchangeActor,
            enablePostQuantum: true,
            enableDaita: false
        ) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        peerExchangeActor
            .delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, ephemeralKey, daita in
                await multiHopPeerExchanger.receivePostQuantumKey(
                    preSharedKey,
                    ephemeralKey: ephemeralKey,
                    daitaParameters: daita
                )
            })
        await multiHopPeerExchanger.start()

        wait(
            for: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testEphemeralPeerExchangeSuccessWhenDaitaNegotiationStarts() async throws {
        let unexpectedNegotiationFailure = expectation(description: "Negotiation failed.")
        unexpectedNegotiationFailure.isInverted = true

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 3

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let peerExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        peerExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let multiHopPeerExchanger = MultiHopEphemeralPeerExchanger(
            entry: entryRelay,
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: peerExchangeActor,
            enablePostQuantum: false,
            enableDaita: true
        ) { params in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        peerExchangeActor.delegate = KeyExchangingResultStub(onReceiveEphemeralPeerPrivateKey: { ephemeralKey, daita in
            await multiHopPeerExchanger.receiveEphemeralPeerPrivateKey(ephemeralKey, daitaParameters: daita)
        })
        await multiHopPeerExchanger.start()

        wait(
            for: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testEphemeralPeerExchangeSuccessPassesDaitaParameters() async throws {
        let unexpectedNegotiationFailure = expectation(description: "Negotiation failed.")
        unexpectedNegotiationFailure.isInverted = true

        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 3

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let peerExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        peerExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let multiHopPeerExchanger = MultiHopEphemeralPeerExchanger(
            entry: entryRelay,
            exit: exitRelay,
            devicePrivateKey: PrivateKey(),
            keyExchanger: peerExchangeActor,
            enablePostQuantum: false,
            enableDaita: true
        ) { params in
            if case let .multi(entry, exit) = params {
                XCTAssertNotNil(entry.configuration.daitaParameters)
                XCTAssertNil(exit.configuration.daitaParameters)
            }
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        peerExchangeActor.delegate = KeyExchangingResultStub(onReceiveEphemeralPeerPrivateKey: { ephemeralKey, daita in
            await multiHopPeerExchanger.receiveEphemeralPeerPrivateKey(ephemeralKey, daitaParameters: daita)
        })
        await multiHopPeerExchanger.start()

        wait(
            for: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }
}
