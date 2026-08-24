// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes

/// A concrete implementation of an access method proxy configuration.
class ProxyConfigurationTester: ProxyConfigurationTesterProtocol {
    private var cancellable: MullvadTypes.Cancellable?
    private let apiProxy: APIQuerying

    init(apiProxy: APIQuerying) {
        self.apiProxy = apiProxy
    }

    func start(configuration: PersistentAccessMethod, completion: @escaping @Sendable (Error?) -> Void) {
        cancellable = apiProxy.checkApiAvailability(retryStrategy: .noRetry, accessMethod: configuration) { success in
            switch success {
            case .success: completion(nil)
            case let .failure(error): completion(error)
            }
        }
    }

    func cancel() {
        cancellable?.cancel()
        cancellable = nil
    }
}
