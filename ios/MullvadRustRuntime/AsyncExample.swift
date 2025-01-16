//
//  AsyncExample.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@_silgen_name("async_finish")
func finishAsyncExample(
    response: SwiftMullvadApiResponse,
    asyncCookie: UnsafeMutableRawPointer
) {
    let pointerClass = Unmanaged<PointerClass>
        .fromOpaque(asyncCookie)
        .takeUnretainedValue()
    let apiResponse = MullvadApiResponse(response: response)

    pointerClass.continuation.resume(returning: apiResponse)
}

public class PointerClass {
    public let continuation: CheckedContinuation<MullvadApiResponse, Never>

    public init(continuation: CheckedContinuation<MullvadApiResponse, Never>) {
        self.continuation = continuation
    }
}
