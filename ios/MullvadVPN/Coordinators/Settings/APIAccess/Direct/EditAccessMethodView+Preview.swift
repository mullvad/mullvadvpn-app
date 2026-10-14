// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadMockData
import MullvadSettings
import MullvadTypes
import SwiftUI

#Preview {
    EditAccessMethodView(
        viewModel: EditAccessMethodViewModel(
            accessMethod: PersistentAccessMethod(
                id: AccessMethodRepository.directId,
                name: "Direct",
                isEnabled: true,
                proxyConfiguration: .direct
            ),
            proxyConfigurationTester: ProxyConfigurationTester(
                apiProxy: APIProxyStub(checkApiAvailabilityResult: .failure(CancellationError()))),
            onChange: { newValue in
                print(newValue)
            })
    )
    .environment(\.locale, Locale(identifier: "de"))
}
