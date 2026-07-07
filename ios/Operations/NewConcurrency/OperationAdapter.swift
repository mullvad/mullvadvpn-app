//
//  OperationAdapter.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

class OperationAdapter<T: AsyncExecutable>: AsyncOperation, @unchecked Sendable {

    let executable: T

    init(_ executable: T) {
        self.executable = executable
    }

    override func main() {
        Task {
            do {
                _ = try await executable.execute()
                finish()
            } catch {
                finish(error: error)
            }
        }
    }
}
