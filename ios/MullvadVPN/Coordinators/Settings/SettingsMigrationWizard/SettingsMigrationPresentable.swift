//
//  SettingsMigrationPresentable.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-05-18.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation
import SwiftUI

protocol SettingsMigrationPresentable {
    var title: String { get }
    var banner: Image? { get }
    var description: [TextItem] { get }
}
