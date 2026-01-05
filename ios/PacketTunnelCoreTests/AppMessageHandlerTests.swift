//
//  AppMessageHandlerTests.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-09-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadTypes
import PacketTunnelCore
import XCTest

@testable import MullvadMockData
@testable import MullvadREST

final class AppMessageHandlerTests: XCTestCase {
    // MARK: APIRequest

    func testHandleAppMessageForSendAPIRequest() async throws {
        let sendRequestExpectation = expectation(description: "Expect sending request")

        let apiRequestProxy = APIRequestProxyStub(sendRequestExpectation: sendRequestExpectation)
        let appMessageHandler = createAppMessageHandler(apiRequestProxy: apiRequestProxy)

        let apiRequest = ProxyAPIRequest(
            id: UUID(),
            request: .getAddressList(.default)
        )

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.sendAPIRequest(apiRequest).encode()
        )

        await fulfillment(of: [sendRequestExpectation], timeout: .UnitTest.timeout)
    }

    func testHandleAppMessageForCancelAPIRequest() async throws {
        let cancelRequestExpectation = expectation(description: "Expect cancelling request")

        let apiRequestProxy = APIRequestProxyStub(cancelRequestExpectation: cancelRequestExpectation)
        let appMessageHandler = createAppMessageHandler(apiRequestProxy: apiRequestProxy)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.cancelAPIRequest(UUID()).encode()
        )

        await fulfillment(of: [cancelRequestExpectation], timeout: .UnitTest.timeout)
    }

    // MARK: Other

    func testHandleAppMessageForTunnelStatus() async throws {
        let stateExpectation = expectation(description: "Expect getting state")

        let actor = PacketTunnelActorStub(stateExpectation: stateExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.getTunnelStatus.encode()
        )

        await fulfillment(of: [stateExpectation], timeout: .UnitTest.timeout)
    }

    func testHandleAppMessageForKeyRotation() async throws {
        let keyRotationExpectation = expectation(description: "Expect key rotation")

        let actor = PacketTunnelActorStub(keyRotationExpectation: keyRotationExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.privateKeyRotation.encode()
        )

        await fulfillment(of: [keyRotationExpectation], timeout: .UnitTest.timeout)
    }

    func testHandleAppMessageForReconnectTunnel() async throws {
        let reconnectExpectation = expectation(description: "Expect reconnecting state")

        let actor = PacketTunnelActorStub(reconnectExpectation: reconnectExpectation)
        let appMessageHandler = createAppMessageHandler(actor: actor)

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
            wireguard: ServerRelaysResponseStubs.sampleRelays.wireguard,
            portConstraint: relayConstraints.port,
            numberOfFailedAttempts: 0
        )

        let selectedRelays = SelectedRelays(
            entry: nil,
            exit: SelectedRelay(
                endpoint: SelectedEndpoint(
                    socketAddress: .ipv4(match.endpoint.ipv4Relay),
                    ipv4Gateway: match.endpoint.ipv4Gateway,
                    ipv6Gateway: match.endpoint.ipv6Gateway,
                    publicKey: match.endpoint.publicKey,
                    obfuscation: .off
                ),
                hostname: match.relay.hostname,
                location: match.location,
                features: nil
            ),
            retryAttempt: 0
        )

        _ = try? await appMessageHandler.handleAppMessage(
            TunnelProviderMessage.reconnectTunnel(.preSelected(selectedRelays)).encode()
        )

        await fulfillment(of: [reconnectExpectation], timeout: .UnitTest.timeout)
    }
}

extension AppMessageHandlerTests {
    func createAppMessageHandler(
        actor: PacketTunnelActorProtocol = PacketTunnelActorStub(),
        apiRequestProxy: APIRequestProxyProtocol = APIRequestProxyStub()
    ) -> AppMessageHandler {
        return AppMessageHandler(
            packetTunnelActor: actor,
            apiRequestProxy: apiRequestProxy
        )
    }
}
