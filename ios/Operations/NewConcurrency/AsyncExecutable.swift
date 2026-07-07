//
//  AsyncExecutable.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public protocol AsyncExecutable: Sendable {
    associatedtype Output

    func execute() async throws -> Output
}
