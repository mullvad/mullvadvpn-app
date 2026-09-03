// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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

    public var digest: String? {
        response.sigsum_digest.map { String(cString: $0) }
    }

    public var timestamp: Int64 {
        response.sigsum_timestamp
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
