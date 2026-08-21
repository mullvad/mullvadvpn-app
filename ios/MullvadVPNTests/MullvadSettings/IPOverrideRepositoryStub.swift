// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
