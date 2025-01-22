//
//  View+Modifier.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-01-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension View {
    func apply<V: View>(@ViewBuilder _ block: (Self) -> V) -> V { block(self) }
}
