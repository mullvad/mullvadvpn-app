//
//  NetworkPathSnapshot.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Network

struct NetworkPathSnapshot: Equatable, Sendable {
    let status: Status
    let interfaces: [InterfaceInfo]
    let gateways: [String]
    let isExpensive: Bool
    let isConstrained: Bool

    enum Status: String, Equatable, Sendable, CustomStringConvertible {
        case satisfied, unsatisfied, requiresConnection, unknown
        var description: String { rawValue }
    }

    struct InterfaceInfo: Equatable, Sendable, CustomStringConvertible {
        let name: String
        let type: NWInterface.InterfaceType
        let index: Int
        var description: String { "\(name) (\(type))" }
    }
}

extension NetworkPathSnapshot {
    init(_ path: NWPath) {
        self.status = Self.mapStatus(path.status)
        self.isExpensive = path.isExpensive
        self.isConstrained = path.isConstrained
        self.interfaces = path.availableInterfaces.map {
            InterfaceInfo(name: $0.name, type: $0.type, index: $0.index)
        }
        self.gateways = path.gateways.compactMap { ep -> String? in
            switch ep {
            case .hostPort(let host, _):
                return "\(host)"
            case .service(let name, let type, let domain, _):
                return "\(name).\(type).\(domain)"
            case .unix(let path):
                return "unix:\(path)"
            case .url(let url):
                return url.absoluteString
            case .opaque(let endpoint):
                return String(describing: endpoint)
            @unknown default:
                return nil
            }
        }
    }

    func diffDescription(from previous: NetworkPathSnapshot?) -> String {
        guard let previous else {
            return "initial — interfaces: \(interfaces.map(\.name)), gateways: \(gateways), status: \(status)"
        }

        var parts: [String] = []
        let prevNames = Set(previous.interfaces.map(\.name))
        let currNames = Set(interfaces.map(\.name))

        let added = currNames.subtracting(prevNames)
        let removed = prevNames.subtracting(currNames)
        if !added.isEmpty { parts.append("+\(added.sorted().joined(separator: ", "))") }
        if !removed.isEmpty { parts.append("-\(removed.sorted().joined(separator: ", "))") }
        if gateways != previous.gateways {
            parts.append("gateways: \(previous.gateways) → \(gateways)")
        }
        if status != previous.status {
            parts.append("status: \(previous.status) → \(status)")
        }

        return parts.isEmpty ? "no change" : parts.joined(separator: ", ")
    }

    private static func mapStatus(_ s: NWPath.Status) -> Status {
        switch s {
        case .satisfied: return .satisfied
        case .unsatisfied: return .unsatisfied
        case .requiresConnection: return .requiresConnection
        @unknown default: return .unknown
        }
    }
}
