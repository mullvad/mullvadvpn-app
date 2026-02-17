//
//  MultiHopEphemeralPeerExchangerTests.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-18.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import WireGuardKitTypes

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
            endpoint: SelectedEndpoint(
                socketAddress: .ipv4(entryMatch.endpoint.ipv4Relay),
                ipv4Gateway: entryMatch.endpoint.ipv4Gateway,
                ipv6Gateway: entryMatch.endpoint.ipv6Gateway,
                publicKey: entryMatch.endpoint.publicKey,
                obfuscation: .off
            ),
            hostname: entryMatch.relay.hostname,
            location: entryMatch.location,
            isOverridden: false,
            features: nil
        )
        exitRelay = SelectedRelay(
            endpoint: SelectedEndpoint(
                socketAddress: .ipv4(exitMatch.endpoint.ipv4Relay),
                ipv4Gateway: exitMatch.endpoint.ipv4Gateway,
                ipv6Gateway: exitMatch.endpoint.ipv6Gateway,
                publicKey: exitMatch.endpoint.publicKey,
                obfuscation: .off
            ),
            hostname: exitMatch.relay.hostname,
            location: exitMatch.location,
            isOverridden: false,
            features: nil
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

        await fulfillment(
            of: [expectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
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

        await fulfillment(
            of: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
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
        ) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        peerExchangeActor.delegate = KeyExchangingResultStub(onReceiveEphemeralPeerPrivateKey: { ephemeralKey, daita in
            await multiHopPeerExchanger.receiveEphemeralPeerPrivateKey(ephemeralKey, daitaParameters: daita)
        })
        await multiHopPeerExchanger.start()

        await fulfillment(
            of: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
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

        await fulfillment(
            of: [unexpectedNegotiationFailure, reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }
}
