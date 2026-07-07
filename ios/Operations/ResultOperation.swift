//
//  ResultOperation.swift
//  Operations
//
//  Created by pronebird on 23/03/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

open class ResultOperation<Success: Sendable>: AsyncOperation, AsyncExecutable, OutputOperation, @unchecked Sendable {

    public typealias CompletionHandler = @Sendable (Result<Success, Error>) -> Void

    private let nslock = NSLock()
    private var _output: Success?
    private var _completionQueue: DispatchQueue?
    private var _completionHandler: CompletionHandler?
    private var pendingFinish = false

    // MARK: - Result

    public var result: Result<Success, Error>? {
        nslock.lock()
        defer { nslock.unlock() }

        if let output = _output {
            return .success(output)
        }

        if let error {
            return .failure(error)
        }

        return nil
    }

    public var output: Success? {
        nslock.lock()
        defer { nslock.unlock() }
        return _output
    }

    // MARK: - Completion

    public var completionQueue: DispatchQueue? {
        get {
            nslock.lock()
            defer { nslock.unlock() }
            return _completionQueue
        }
        set {
            nslock.lock()
            defer { nslock.unlock() }
            _completionQueue = newValue
        }
    }

    public var completionHandler: CompletionHandler? {
        get {
            nslock.lock()
            defer { nslock.unlock() }
            return _completionHandler
        }
        set {
            nslock.lock()
            defer { nslock.unlock() }

            guard !pendingFinish else { return }
            _completionHandler = newValue
        }
    }

    // MARK: - Initializers

    override public init(dispatchQueue: DispatchQueue?) {
        super.init(dispatchQueue: dispatchQueue)
    }

    public init(
        dispatchQueue: DispatchQueue?,
        completionQueue: DispatchQueue?,
        completionHandler: CompletionHandler?
    ) {
        _completionQueue = completionQueue
        _completionHandler = completionHandler

        super.init(dispatchQueue: dispatchQueue)
    }

    // MARK: - AsyncExecutable

    /// Subclasses should override this instead of `main()`.
    open func execute() async throws -> Success {
        fatalError("Subclasses must override execute()")
    }

    // MARK: - AsyncOperation

    open override func main() {
        Task {
            do {
                let value = try await execute()
                finish(result: .success(value))
            } catch {
                finish(result: .failure(error))
            }
        }
    }

    // Prevent subclasses from using the old API.
    @available(*, unavailable, message: "Override execute() instead.")
    override public func finish() {
        fatalError()
    }

    @available(*, unavailable, message: "Override execute() instead.")
    override public func finish(error: Error?) {
        fatalError()
    }

    // MARK: - Finish

    public final func finish(result: Result<Success, Error>) {
        nslock.lock()

        guard !pendingFinish else {
            nslock.unlock()
            return
        }

        pendingFinish = true

        let completionHandler = _completionHandler
        _completionHandler = nil

        let completionQueue = _completionQueue

        if case let .success(output) = result {
            _output = output
        }

        nslock.unlock()

        dispatchAsync(on: completionQueue) {
            completionHandler?(result)

            switch result {
            case .success:
                super.finish(error: nil)

            case .failure(let error):
                super.finish(error: error)
            }
        }
    }

    // MARK: - Helpers

    private func dispatchAsync(
        on queue: DispatchQueue?,
        _ block: @escaping @Sendable () -> Void
    ) {
        if let queue {
            queue.async(execute: block)
        } else {
            block()
        }
    }
}
