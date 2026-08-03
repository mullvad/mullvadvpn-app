//
//  AsyncSerialExecutor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-03.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public actor AsyncSerialExecutor {
    private var previousTask: Task<Void, Never>?

    public init() {}

    public func execute<T: Sendable>(_ job: @escaping @Sendable () async throws -> T) async throws -> T {
        let previousTask = self.previousTask

        let task = Task {
            await previousTask?.value
            return try await job()
        }

        self.previousTask = Task {
            _ = try? await task.value
        }

        return try await task.value
    }
}
