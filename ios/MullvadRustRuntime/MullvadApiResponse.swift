//
//  MullvadApiResponse.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
        guard let body = response.body else {
            return nil
        }

        return Data(UnsafeBufferPointer(start: body, count: Int(response.body_size)))
    }

    public var etag: String? {
        response.etag.map { String(cString: $0) }
    }

    public var errorDescription: String? {
        response.error_description.map { String(cString: $0) }
    }

    public var statusCode: UInt16 {
        response.status_code
    }

    public var serverResponseCode: String? {
        response.server_response_code.map { String(cString: $0) }
    }

    public var success: Bool {
        response.success
    }
}
