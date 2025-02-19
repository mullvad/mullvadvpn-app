//
//  MullvadApiResponse.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

public class MullvadApiResponse {
    private let response: SwiftMullvadApiResponse

    public init(response: consuming SwiftMullvadApiResponse) {
        self.response = response
    }

    deinit {
        mullvad_response_drop(response)
    }

    public var body: Data? {
        Data(UnsafeBufferPointer(start: response.body, count: Int(response.body_size)))
    }

    public var errorDescription: String? {
        return if response.error_description == nil {
            nil
        } else {
            String(cString: response.error_description)
        }
    }

    public var statusCode: UInt16 {
        response.status_code
    }

    public var success: Bool {
        response.success
    }
}
