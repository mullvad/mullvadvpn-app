//
//  Separator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-20.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RowSeparator: View {
    var color: Color = Color(.secondaryColor)

    var body: some View {
        color
            .frame(height: UIMetrics.TableView.separatorHeight)
            .padding(.leading, 16)
    }
}

#Preview {
    RowSeparator(color: Color(.primaryColor))
}
