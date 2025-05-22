//
//  IPOverrideRepositoryStub.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-01-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@preconcurrency import Combine
import MullvadSettings

struct IPOverrideRepositoryStub: IPOverrideRepositoryProtocol {
    let passthroughSubject: CurrentValueSubject<[IPOverride], Never> = CurrentValueSubject([])
    var overridesPublisher: AnyPublisher<[IPOverride], Never> {
        passthroughSubject.eraseToAnyPublisher()
    }

    let overrides: [IPOverride]

    init(overrides: [IPOverride] = []) {
        self.overrides = overrides
    }

    func add(_ overrides: [IPOverride]) {}

    func fetchAll() -> [IPOverride] {
        overrides
    }

    func fetchByHostname(_ hostname: String) -> IPOverride? {
        nil
    }

    func deleteAll() {}

    func parse(data: Data) throws -> [IPOverride] {
        overrides
    }
}
