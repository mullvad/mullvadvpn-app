// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public protocol APITransportProviderProtocol {
    func makeTransport() -> APITransportProtocol?
}

public final class APITransportProvider: APITransportProviderProtocol, Sendable {
    let requestFactory: MullvadApiRequestFactory

    public init(requestFactory: MullvadApiRequestFactory) {
        self.requestFactory = requestFactory
    }

    public func makeTransport() -> APITransportProtocol? {
        APITransport(requestFactory: requestFactory)
    }
}

extension REST {
    public struct AnyAPITransportProvider: APITransportProviderProtocol {
        private let block: () -> APITransportProtocol?

        public init(_ block: @escaping @Sendable () -> APITransportProtocol?) {
            self.block = block
        }

        public func makeTransport() -> APITransportProtocol? {
            block()
        }
    }
}
