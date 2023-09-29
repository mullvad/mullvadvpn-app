//
//  CommandChannel.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/**
 Sync-to-async ordered coalescing channel with unbound buffering.

 Publishers send commands over the channel to pass work to consumer. Received commands are buffered, until requested by consumer and coalesced just
 before consumption.

 - Multiple consumers are possible but the actor is really expected to be the only consumer.
 - Internally, the channel acquires a lock, so you can assume FIFO ordering unless you publish values simultaneously from multiple concurrent queues.

 ### Example

 ```
 let channel = CommandChannel()
 channel.send(.stop)
 ```

 Consuming commands can be implemented using a for-await loop. Note that using a loop should also serialize the command handling as the next command will not
 be consumed until the body of the loop completes the iteration.

 ```
 Task.detached {
     for await command in channel {
        await handleMyCommand(command)
     }
 }
 ```

 Normally channel is expected to be infinite, but it's convenient to end the stream earlier, for instance when testing the coalescing algorithm:

 ```
 channel.send(.start(..))
 channel.send(.stop)
 channel.sendEnd()

 let allReceivedCommands = channel
     .map { "\($0)" }
     .reduce(into: [String]()) { $0.append($1) }
 ```
 */
final class CommandChannel: @unchecked Sendable {
    private enum State {
        /// Channel is active and running.
        case active

        /// Channel is awaiting for the buffer to be exhausted before ending all async iterations.
        /// Publishing new values in this state is impossible.
        case pendingEnd

        /// Channel finished its work.
        /// Publishing new values in this state is impossible.
        /// An attempt to iterate over the channel in this state is equivalent to iterating over an empty array.
        case finished
    }

    /// A buffer of commands received but not consumed yet.
    private var buffer: [Command] = []

    /// Async continuations awaiting to receive the new value.
    /// Continuations are stored here when there is no new value available for immediate delivery.
    private var pendingContinuations: [CheckedContinuation<Command?, Never>] = []

    private var state: State = .active
    private var stateLock = NSLock()

    init() {}

    deinit {
        // Resume all continuations
        finish()
    }

    /// Send command to consumer.
    ///
    /// - Parameter value: a new command.
    func send(_ value: Command) {
        stateLock.withLock {
            guard case .active = state else { return }

            buffer.append(value)

            if !pendingContinuations.isEmpty, let nextValue = consumeFirst() {
                let continuation = pendingContinuations.removeFirst()
                continuation.resume(returning: nextValue)
            }
        }
    }

    /// Mark the end of channel but let consumers exchaust the buffer before declaring the end of iteration.
    /// If the buffer is empty then it should resume all pending continuations and send them `nil` to mark the end of iteration.
    func sendEnd() {
        stateLock.withLock {
            if case .active = state {
                state = .pendingEnd

                if buffer.isEmpty {
                    state = .finished
                    sendEndToPendingContinuations()
                }
            }
        }
    }

    /// Flush buffered commands and resume all pending continuations sending them `nil` to mark the end of iteration.
    func finish() {
        stateLock.withLock {
            switch state {
            case .active, .pendingEnd:
                state = .finished
                buffer.removeAll()

                sendEndToPendingContinuations()

            case .finished:
                break
            }
        }
    }

    /// Send `nil` to mark the end of iteration to all pending continuations.
    private func sendEndToPendingContinuations() {
        for continuation in pendingContinuations {
            continuation.resume(returning: nil)
        }
        pendingContinuations.removeAll()
    }

    /// Consume first message in the buffer.
    /// Returns `nil` if the buffer is empty, otherwise if attempts to coalesce buffered commands before consuming the first comand in the list.
    private func consumeFirst() -> Command? {
        guard !buffer.isEmpty else { return nil }

        coalesce()
        return buffer.removeFirst()
    }

    /// Coalesce buffered commands to prevent future execution when the outcome is considered to be similar.
    /// Mutates internal `buffer`.
    private func coalesce() {
        var i = buffer.count - 1
        while i > 0 {
            defer { i -= 1 }

            assert(i < buffer.count)
            let current = buffer[i]

            // Remove all preceding commands when encountered "stop".
            if case .stop = current {
                buffer.removeFirst(i)
                return
            }

            // Coalesce earlier reconnection attempts into the most recent.
            // This will rearrange the command buffer but hopefully should have no side effects.
            if case .reconnect = current {
                // Walk backwards starting with the preceding element.
                for j in (0 ..< i).reversed() {
                    let preceding = buffer[j]
                    // Remove preceding reconnect and adjust the index of the outer loop.
                    if case .reconnect = preceding {
                        buffer.remove(at: j)
                        i -= 1
                    }
                }
            }
        }
    }

    private func next() async -> Command? {
        return await withCheckedContinuation { continuation in
            stateLock.withLock {
                switch state {
                case .pendingEnd:
                    if buffer.isEmpty {
                        state = .finished
                        continuation.resume(returning: nil)
                    } else {
                        // Keep consuming until the buffer is exhausted.
                        fallthrough
                    }

                case .active:
                    if let value = consumeFirst() {
                        continuation.resume(returning: value)
                    } else {
                        pendingContinuations.append(continuation)
                    }

                case .finished:
                    continuation.resume(returning: nil)
                }
            }
        }
    }
}

extension CommandChannel: AsyncSequence {
    typealias Element = Command

    struct AsyncIterator: AsyncIteratorProtocol {
        let channel: CommandChannel
        func next() async -> Command? {
            return await channel.next()
        }
    }

    func makeAsyncIterator() -> AsyncIterator {
        return AsyncIterator(channel: self)
    }
}
