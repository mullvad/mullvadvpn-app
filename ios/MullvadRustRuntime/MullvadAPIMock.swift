// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadRustRuntimeProxy

public class MullvadApiMock {
    private let mock: ServerMock

    public var port: UInt16 {
        mock.port()
    }

    private init(_ mock: ServerMock) {
        self.mock = mock
    }

    public static func get(path: String, responseCode: UInt, responseData: String) -> MullvadApiMock {
        let endpoint = MockEndpoint(path: path, responseCode: UInt64(responseCode), responseBody: responseData)
        return .init(ServerMock.get(endpoints: [endpoint]))
    }

    public static func get(_ endpoints: [(path: String, responseCode: UInt, responseData: String)]) -> MullvadApiMock {
        let endpoints = endpoints.map { p in
            MockEndpoint(path: p.path, responseCode: UInt64(p.responseCode), responseBody: p.responseData)
        }
        return .init(ServerMock.get(endpoints: endpoints))
    }

    public static func post(path: String, responseCode: UInt, responseData: String) -> MullvadApiMock {
        .init(ServerMock.post(path: path, responseCode: UInt64(responseCode), matchBody: responseData))
    }
}
