// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

@_cdecl("mullvad_api_completion_finish")
func mullvadApiCompletionFinish(
    response: SwiftMullvadApiResponse,
    completionCookie: UnsafeMutableRawPointer
) {
    let completionBridge = Unmanaged<MullvadApiCompletion>
        .fromOpaque(completionCookie)
        .takeRetainedValue()
    let apiResponse = MullvadApiResponse(response: response)

    completionBridge.completion(apiResponse)
}

public class MullvadApiCompletion {
    public var completion: (MullvadApiResponse) -> Void

    public init(completion: @escaping ((MullvadApiResponse) -> Void)) {
        self.completion = completion
    }
}
