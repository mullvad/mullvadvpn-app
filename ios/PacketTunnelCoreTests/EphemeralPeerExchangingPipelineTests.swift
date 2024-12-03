//
//  EphemeralPeerExchangingPipelineTests.swift
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

final class EphemeralPeerExchangingPipelineTests: XCTestCase {
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

    func testSingleHopPostQuantumKeyExchange() throws {
        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 2

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let postQuantumKeyExchangingPipeline = EphemeralPeerExchangingPipeline(keyExchangeActor) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, privateKey in
            postQuantumKeyExchangingPipeline.receivePostQuantumKey(preSharedKey, ephemeralKey: privateKey)
        })

        let connectionState = stubConnectionState(enableMultiHop: false, enablePostQuantum: true, enableDaita: false)
        postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        wait(
            for: [reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testSingleHopDaitaPeerExchange() throws {
        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 2

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let postQuantumKeyExchangingPipeline = EphemeralPeerExchangingPipeline(keyExchangeActor) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceiveEphemeralPeerPrivateKey: { privateKey in
            postQuantumKeyExchangingPipeline.receiveEphemeralPeerPrivateKey(privateKey)
        })

        let connectionState = stubConnectionState(enableMultiHop: false, enablePostQuantum: false, enableDaita: true)
        postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        wait(
            for: [reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testMultiHopPostQuantumKeyExchange() throws {
        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 3

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let postQuantumKeyExchangingPipeline = EphemeralPeerExchangingPipeline(keyExchangeActor) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, privateKey in
            postQuantumKeyExchangingPipeline.receivePostQuantumKey(preSharedKey, ephemeralKey: privateKey)
        })

        let connectionState = stubConnectionState(enableMultiHop: true, enablePostQuantum: true, enableDaita: false)
        postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        wait(
            for: [reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func testMultiHopDaitaExchange() throws {
        let reconfigurationExpectation = expectation(description: "Tunnel reconfiguration took place")
        reconfigurationExpectation.expectedFulfillmentCount = 3

        let negotiationSuccessful = expectation(description: "Negotiation succeeded.")
        negotiationSuccessful.expectedFulfillmentCount = 1

        let keyExchangeActor = EphemeralPeerExchangeActorStub()
        let preSharedKey = try XCTUnwrap(PreSharedKey(hexKey: PrivateKey().hexKey))
        keyExchangeActor.result = .success((preSharedKey, PrivateKey()))

        let postQuantumKeyExchangingPipeline = EphemeralPeerExchangingPipeline(keyExchangeActor) { _ in
            reconfigurationExpectation.fulfill()
        } onFinish: {
            negotiationSuccessful.fulfill()
        }

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceiveEphemeralPeerPrivateKey: { privateKey in
            postQuantumKeyExchangingPipeline.receiveEphemeralPeerPrivateKey(privateKey)
        })

        let connectionState = stubConnectionState(enableMultiHop: true, enablePostQuantum: false, enableDaita: true)
        postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        wait(
            for: [reconfigurationExpectation, negotiationSuccessful],
            timeout: .UnitTest.invertedTimeout
        )
    }

    func stubConnectionState(
        enableMultiHop: Bool,
        enablePostQuantum: Bool,
        enableDaita: Bool
    ) -> ObservedConnectionState {
        ObservedConnectionState(
            selectedRelays: SelectedRelays(entry: enableMultiHop ? entryRelay : nil, exit: exitRelay, retryAttempt: 0),
            relayConstraints: relayConstraints,
            networkReachability: NetworkReachability.reachable,
            connectionAttemptCount: 0,
            transportLayer: .udp,
            remotePort: 1234,
            isPostQuantum: enablePostQuantum,
            isDaitaEnabled: enableDaita
        )
    }
}
