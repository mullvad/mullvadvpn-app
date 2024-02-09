//
//  SelectLocationNodeProtocol.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

protocol SelectLocationNodeProtocol {
    var location: RelayLocation { get }
    var displayName: String { get }
    var showsChildren: Bool { get }
    var isActive: Bool { get }
    var isCollapsible: Bool { get }
    var indentationLevel: Int { get }
}
