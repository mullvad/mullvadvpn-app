//
//  MullvadApiCompletion.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public final class MullvadApiCompletion: CompletionCookieNew {
    public let completion: @Sendable (SwiftMullvadApiResponse) -> Void

    public init(completion: @Sendable @escaping (SwiftMullvadApiResponse) -> Void) {
        self.completion = completion
    }

    public func finish(result: SwiftMullvadApiResponse) {
        self.completion(result)
    }
}
