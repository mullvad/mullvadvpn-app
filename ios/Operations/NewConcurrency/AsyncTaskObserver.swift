//
//  AsyncTaskObserver.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public protocol AsyncTaskObserver: Sendable {
    func didStart()
    func didCancel()
    func didFinish(error: Error?)
}

public extension AsyncTaskObserver {
    func didStart() {}
    func didCancel() {}
    func didFinish(error: Error?) {}
}
