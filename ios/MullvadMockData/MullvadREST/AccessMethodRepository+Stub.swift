//
//  AccessMethodRepository+Stub.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes

public struct AccessMethodRepositoryStub: AccessMethodRepositoryDataSource, @unchecked Sendable {
    public let shadowsocksCiphers: [String] = []
    public var directAccess: PersistentAccessMethod

    public var accessMethodsPublisher: AnyPublisher<[PersistentAccessMethod], Never> {
        passthroughSubject.eraseToAnyPublisher()
    }

    let passthroughSubject: CurrentValueSubject<[PersistentAccessMethod], Never> = CurrentValueSubject([])

    public init(accessMethods: [PersistentAccessMethod]) {
        directAccess = accessMethods.first(where: { $0.kind == .direct })!
        passthroughSubject.send(accessMethods)
    }

    public func fetchAll() -> [PersistentAccessMethod] {
        passthroughSubject.value
    }

    public func requestAccessMethod(_ method: PersistentAccessMethod) {}

    public func fetchLastReachable() -> PersistentAccessMethod {
        directAccess
    }

    public func infoHeaderConfig(for id: UUID) -> InfoHeaderConfig? {
        nil
    }

    public static var stub: AccessMethodRepositoryStub {
        AccessMethodRepositoryStub(accessMethods: [
            PersistentAccessMethod(
                id: UUID(),
                name: "Direct",
                isEnabled: true,
                proxyConfiguration: .direct
            ),
            PersistentAccessMethod(
                id: UUID(),
                name: "Bridges",
                isEnabled: true,
                proxyConfiguration: .bridges
            ),
            PersistentAccessMethod(
                id: UUID(),
                name: "Encrypted DNS",
                isEnabled: true,
                proxyConfiguration: .encryptedDNS
            ),
            PersistentAccessMethod(
                id: UUID(),
                name: "Domain fronting",
                isEnabled: true,
                proxyConfiguration: .domainFronting
            ),
        ])
    }
}
