//
//  CustomListLocationNode.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-01.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadSettings
import MullvadTypes

class CustomListLocationNode: LocationNode, @unchecked Sendable {
    let customList: CustomList

    init(
        name: String,
        code: String,
        locations: [RelayLocation] = [],
        isActive: Bool = true,
        parent: LocationNode? = nil,
        children: [LocationNode] = [],
        showsChildren: Bool = false,
        isHiddenFromSearch: Bool = false,
        customList: CustomList
    ) {
        self.customList = customList

        super.init(
            name: name,
            code: code,
            locations: locations,
            isActive: isActive,
            parent: parent,
            children: children,
            showsChildren: showsChildren,
            isHiddenFromSearch: isHiddenFromSearch
        )
    }
}
