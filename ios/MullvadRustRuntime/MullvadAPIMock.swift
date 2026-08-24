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
    private let mock: SwiftServerMock

    public var port: UInt16 {
        mock.port
    }

    private init(_ mock: SwiftServerMock) {
        self.mock = mock
    }

    public static func get(path: String, responseCode: UInt, responseData: String) -> MullvadApiMock {
        let newMock = mullvad_api_mock_get(path, responseCode, responseData)
        return MullvadApiMock(newMock)
    }

    public static func post(path: String, responseCode: UInt, responseData: String) -> MullvadApiMock {
        let newMock = mullvad_api_mock_post(path, responseCode, responseData)
        return MullvadApiMock(newMock)
    }

    deinit {
        mullvad_api_mock_drop(self.mock)
    }
}
