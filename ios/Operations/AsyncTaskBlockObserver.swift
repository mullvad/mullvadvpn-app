//
//  AsyncTaskBlockObserver.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

public final class AsyncTaskBlockObserver: AsyncTaskObserver, @unchecked Sendable {
    public typealias VoidBlock = @Sendable () -> Void
    public typealias FinishBlock = @Sendable (Error?) -> Void

    private let didStartBlock: VoidBlock?
    private let didCancelBlock: VoidBlock?
    private let didFinishBlock: FinishBlock?

    public init(
        didStart: VoidBlock? = nil,
        didCancel: VoidBlock? = nil,
        didFinish: FinishBlock? = nil
    ) {
        self.didStartBlock = didStart
        self.didCancelBlock = didCancel
        self.didFinishBlock = didFinish
    }

    public func didStart() {
        didStartBlock?()
    }

    public func didCancel() {
        didCancelBlock?()
    }

    public func didFinish(error: Error?) {
        didFinishBlock?(error)
    }
}
