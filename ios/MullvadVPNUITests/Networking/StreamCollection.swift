//
//  StreamCollection.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-06-24.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

enum StreamCollectionError: Error {
    case noMatchingStreams
}

class StreamCollection {
    private var streams: [Stream] = []

    init(streams: [Stream]) {
        self.streams = streams
    }

    public func getConnectedThroughRelayDateInterval(relayIPAddress: String) throws -> DateInterval {
        return try getDateIntervalOfUDPCommunicationFromTestDevice(toHost: relayIPAddress)
    }

    public func getStreams(between host1: String, host2: String) -> [Stream] {
        let matchingStreams = streams.filter {
            ($0.sourceAddress == host1 && $0.destinationAddress == host2) ||
                ($0.sourceAddress == host2 && $0.destinationAddress == host1)
        }

        return matchingStreams
    }

    public func extractStreamCollectionFrom(
        _ dateInterval: DateInterval,
        cutOffPacketsOverflow: Bool = false
    ) -> StreamCollection {
        let matchingStreams = streams.filter { stream in
            return stream.dateInterval.end > dateInterval.start &&
                stream.dateInterval.start < dateInterval.end
        }

        // Trim overflowing packets if specified
        if cutOffPacketsOverflow {
            for stream in matchingStreams where stream.dateInterval.end < dateInterval.end {
                if stream.dateInterval.end < dateInterval.end {
                    for packet in stream.packets {
                        stream.removePacket(packet: packet)
                    }
                }
            }
        }

        return StreamCollection(streams: matchingStreams)
    }

    public func verifyDontHaveStreams() {
        XCTAssertEqual(streams.count, 0)
    }

    public func verifyDontHaveStreams(exceptToIPAddress: String) {
        for stream in streams {
            XCTAssertEqual(stream.destinationAddress, exceptToIPAddress, "Traffic destined to excpected IP address")
        }
    }

    public func getLeakCount() -> Int {
        var leakCount = 0
        var acceptableLeakCount = 0

        for stream in streams {
            leakCount += stream.getLeakCount()
            acceptableLeakCount += stream.getAcceptableLeakCount()
        }

        return leakCount + acceptableLeakCount
    }

    public func verifyDontHaveLeaks() {
        XCTAssertEqual(getLeakCount(), 0)
    }

    public func allowTrafficFromTestDevice(to: String) {
        for stream in streams {
            for packet in stream.packets {
                if packet.fromPeer && stream.destinationAddress == to {
                    packet.leakStatus = .noLeak
                }
            }
        }
    }

    public func dontAllowTrafficFromTestDevice(to: String) {
        for stream in streams {
            for packet in stream.packets {
                if packet.fromPeer && stream.destinationAddress == to {
                    packet.leakStatus = .leak
                }
            }
        }
    }

    public func allowTrafficBefore(_ date: Date) {
        let streamsWithEarlierStartDate = streams.filter { $0.dateInterval.start < date }

        for stream in streamsWithEarlierStartDate {
            for packet in stream.packets {
                if packet.date <= date {
                    packet.leakStatus = .noLeak
                }
            }
        }
    }

    public func allowTrafficAfter(_ date: Date) {
        let streamsWithLaterEndDate = streams.filter { $0.dateInterval.end > date }

        for stream in streamsWithLaterEndDate {
            for packet in stream.packets {
                if packet.date >= date {
                    packet.leakStatus = .noLeak
                }
            }
        }
    }

    private func getDateIntervalOfUDPCommunicationFromTestDevice(toHost: String) throws -> DateInterval {
        var startDate: Date?
        var endDate: Date?

        let matchingStreams = streams.filter { $0.destinationAddress == toHost && $0.transportProtocol == .UDP }

        if matchingStreams.isEmpty {
            throw StreamCollectionError.noMatchingStreams
        }

        for stream in matchingStreams {
            let matchingPackets = stream.packets.filter { $0.fromPeer }.sorted { $0.date < $1.date }

            if let firstMatchingPacket = matchingPackets.first, let lastMatchingPacket = matchingPackets.last {
                if startDate == nil {
                    startDate = firstMatchingPacket.date
                } else {
                    if firstMatchingPacket.date < startDate! {
                        startDate = firstMatchingPacket.date
                    }
                }

                if endDate == nil {
                    endDate = lastMatchingPacket.date
                } else {
                    if lastMatchingPacket.date > endDate! {
                        endDate = lastMatchingPacket.date
                    }
                }
            }
        }

        return DateInterval(start: startDate!, end: endDate!)
    }
}
