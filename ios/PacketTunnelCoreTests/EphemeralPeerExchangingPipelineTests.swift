import XCTest

//
//  EphemeralPeerExchangingPipelineTests.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import PacketTunnelCore
@testable import WireGuardKitTypes

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
                filterConstraint: relayConstraints.exitFilter,
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
                filterConstraint: relayConstraints.entryFilter,
                daitaEnabled: false
            ),
            wireguard: ServerRelaysResponseStubs.sampleRelays.wireguard,
            portConstraint: relayConstraints.port,
            numberOfFailedAttempts: 0
        )

        entryRelay = SelectedRelay(
            endpoint: entryMatch.endpoint,
            hostname: entryMatch.relay.hostname,
            location: entryMatch.location,
            features: nil
        )
        exitRelay = SelectedRelay(
            endpoint: exitMatch.endpoint,
            hostname: exitMatch.relay.hostname,
            location: exitMatch.location,
            features: nil
        )
    }

    func testSingleHopPostQuantumKeyExchange() async throws {
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

        keyExchangeActor
            .delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, privateKey, daita in
                await postQuantumKeyExchangingPipeline.receivePostQuantumKey(
                    preSharedKey,
                    ephemeralKey: privateKey,
                    daitaParameters: daita
                )
            })

        let connectionState = stubConnectionState(enableMultiHop: false, enablePostQuantum: true, enableDaita: false)
        await postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        await fulfillment(of: [reconfigurationExpectation, negotiationSuccessful], timeout: .UnitTest.invertedTimeout)
    }

    func testSingleHopDaitaPeerExchange() async throws {
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

        keyExchangeActor
            .delegate = KeyExchangingResultStub(onReceiveEphemeralPeerPrivateKey: { privateKey, daitaParameters in
                await postQuantumKeyExchangingPipeline.receiveEphemeralPeerPrivateKey(
                    privateKey,
                    daitaParameters: daitaParameters
                )
            })

        let connectionState = stubConnectionState(enableMultiHop: false, enablePostQuantum: false, enableDaita: true)
        await postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        await fulfillment(of: [reconfigurationExpectation, negotiationSuccessful], timeout: .UnitTest.invertedTimeout)
    }

    func testMultiHopPostQuantumKeyExchange() async throws {
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

        keyExchangeActor
            .delegate = KeyExchangingResultStub(onReceivePostQuantumKey: { preSharedKey, privateKey, daita in
                await postQuantumKeyExchangingPipeline.receivePostQuantumKey(
                    preSharedKey,
                    ephemeralKey: privateKey,
                    daitaParameters: daita
                )
            })

        let connectionState = stubConnectionState(enableMultiHop: true, enablePostQuantum: true, enableDaita: false)
        await postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        await fulfillment(of: [reconfigurationExpectation, negotiationSuccessful], timeout: .UnitTest.invertedTimeout)
    }

    func testMultiHopDaitaExchange() async throws {
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

        keyExchangeActor.delegate = KeyExchangingResultStub(onReceiveEphemeralPeerPrivateKey: { privateKey, daita in
            await postQuantumKeyExchangingPipeline.receiveEphemeralPeerPrivateKey(privateKey, daitaParameters: daita)
        })

        let connectionState = stubConnectionState(enableMultiHop: true, enablePostQuantum: false, enableDaita: true)
        await postQuantumKeyExchangingPipeline.startNegotiation(connectionState, privateKey: PrivateKey())

        await fulfillment(of: [reconfigurationExpectation, negotiationSuccessful], timeout: .UnitTest.invertedTimeout)
    }

    func stubConnectionState(
        enableMultiHop: Bool,
        enablePostQuantum: Bool,
        enableDaita: Bool
    ) -> ObservedConnectionState {
        ObservedConnectionState(
            selectedRelays: SelectedRelays(
                entry: enableMultiHop ? entryRelay : nil,
                exit: exitRelay,
                retryAttempt: 0,
                obfuscation: .off
            ),
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
