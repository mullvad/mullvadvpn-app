//
//  PostQuantumKeyExchangingPipelineTests.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//
@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import PacketTunnelCore
@testable import WireGuardKitTypes
import XCTest

final class PostQuantumKeyExchangingPipelineTests: XCTestCase {
    var entryRelay: SelectedRelay!
    var exitRelay: SelectedRelay!
    var relayConstraints: RelayConstraints!

    override func setUpWithError() throws {
        relayConstraints = RelayConstraints(
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

    func testSingleHopKeyExchange() throws {
        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 2

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = PostQuantumKeyExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let postQuantumKeyExchangingPipeline = PostQuantumKeyExchangingPipeline(keyExchangeActor) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, privateKey in
            postQuantumKeyExchangingPipeline.receivePostQuantumKey(preSharedKey, ephemeralKey: privateKey)
        })

        let connectionState = ObservedConnectionState(
            selectedRelays: SelectedRelays(entry: nil, exit: exitRelay, retryAttempt: 0),
            relayConstraints: relayConstraints,
            networkReachability: NetworkReachability.reachable,
            connectionAttemptCount: 0,
            transportLayer: .udp,
            remotePort: 1234,
            isPostQuantum: true
        )

        postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        wait(
            for: [reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testMultiHopKeyExchange() throws {
        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 3

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = PostQuantumKeyExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let postQuantumKeyExchangingPipeline = PostQuantumKeyExchangingPipeline(keyExchangeActor) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, privateKey in
            postQuantumKeyExchangingPipeline.receivePostQuantumKey(preSharedKey, ephemeralKey: privateKey)
        })

        let connectionState = ObservedConnectionState(
            selectedRelays: SelectedRelays(entry: entryRelay, exit: exitRelay, retryAttempt: 0),
            relayConstraints: relayConstraints,
            networkReachability: NetworkReachability.reachable,
            connectionAttemptCount: 0,
            transportLayer: .udp,
            remotePort: 1234,
            isPostQuantum: true
        )

        postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        wait(
            for: [reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }
}
