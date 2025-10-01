//
//  LocationNodeRepresentable.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import MullvadTypes
protocol LocationNodeRepresentable: Identifiable {
    var name: String { get }
    var code: String { get }
    var isActive: Bool { get }
}

protocol ExpandableLocationNodeRepresentable: LocationNodeRepresentable {
    associatedtype Child = Self
    var parent: Child? { get }
    var children: [Child] { get }
    var showsChildren: Bool { get }
    var isHiddenFromSearch: Bool { get }
}

protocol RecentsLocationNodeRepresentable: LocationNodeRepresentable {
    var entryLocation: LocationNode? { get }
    var exitLocation: LocationNode { get }
}
