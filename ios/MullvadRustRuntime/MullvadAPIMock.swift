//
//  MullvadAPIMock.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-11-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
