// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes

actor ProxyConfigurationTesterMock: ProxyConfigurationTesterProtocol {

    private(set) var testedConfigurations: [PersistentAccessMethod] = []
    private(set) var cancelCallCount = 0

    func start(configuration: PersistentAccessMethod) async throws {
        testedConfigurations.append(configuration)
    }

    func cancel() async {
        cancelCallCount += 1
    }
}
