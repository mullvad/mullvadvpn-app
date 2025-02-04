//
//  EventChannel.swift
//  PacketTunnelCore
//
//  Created by pronebird on 27/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//  Formerly known as CommandChannel
//

import Foundation

/**
 Sync-to-async ordered coalescing channel with unbound buffering.

 Publishers send events over the channel to pass work to consumer. Received events
 are buffered, until requested by consumer and coalesced just before consumption.

 - Multiple consumers are possible but the actor is really expected to be the only consumer.
 - Internally, the channel acquires a lock, so you can assume FIFO ordering unless you publish values simultaneously from multiple concurrent queues.

 ### Example

 ```
 let channel = EventChannel()
 channel.send(.stop)
 ```

 Consuming events can be implemented using a for-await loop. Note that using a loop should also serialize the event handling as the next event will not
 be consumed until the body of the loop completes the iteration.

 ```
 Task.detached {
     for await event in channel {
        await handleMyEvent(event)
     }
 }
 ```

 Normally channel is expected to be infinite, but it's convenient to end the stream earlier, for instance when testing the coalescing algorithm:

 ```
 channel.send(.start(..))
 channel.send(.stop)
 channel.sendEnd()

 let allReceivedEvents = channel
     .map { "\($0)" }
     .reduce(into: [String]()) { $0.append($1) }
 ```
 */
extension PacketTunnelActor {
    final class EventChannel: @unchecked Sendable {
        typealias Event = PacketTunnelActor.Event
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

        /// A buffer of events received but not consumed yet.
        private var buffer: [Event] = []

        /// Async continuations awaiting to receive the new value.
        /// Continuations are stored here when there is no new value available for immediate delivery.
        private var pendingContinuations: [CheckedContinuation<Event?, Never>] = []

        private var state: State = .active
        private var stateLock = NSLock()

        init() {}

        deinit {
            // Resume all continuations
            finish()
        }

        /// Send event to consumer.
        ///
        /// - Parameter value: a new event.
        func send(_ value: Event) {
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

        /// Flush buffered events and resume all pending continuations sending them `nil` to mark the end of iteration.
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
        /// Returns `nil` if the buffer is empty, otherwise if attempts to coalesce buffered events before consuming the first comand in the list.
        private func consumeFirst() -> Event? {
            guard !buffer.isEmpty else { return nil }

            coalesce()
            return buffer.removeFirst()
        }

        /// Coalesce buffered events to prevent future execution when the outcome is considered to be similar.
        /// Mutates internal `buffer`.
        private func coalesce() {
            var i = buffer.count - 1
            while i > 0 {
                defer { i -= 1 }

                assert(i < buffer.count)
                let current = buffer[i]

                // Remove all preceding events when encountered "stop".
                if case .stop = current {
                    buffer.removeFirst(i)
                    return
                }

                // Coalesce earlier reconnection attempts into the most recent.
                // This will rearrange the event buffer but hopefully should have no side effects.
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

        private func next() async -> Event? {
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
}

extension PacketTunnelActor.EventChannel: AsyncSequence {
    typealias Element = Event

    struct AsyncIterator: AsyncIteratorProtocol {
        let channel: PacketTunnelActor.EventChannel
        func next() async -> Event? {
            return await channel.next()
        }
    }

    func makeAsyncIterator() -> AsyncIterator {
        return AsyncIterator(channel: self)
    }
}
