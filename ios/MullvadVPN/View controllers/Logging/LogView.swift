//
//  LogView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct LogView: View {
    @ObservedObject var viewModel: LogViewModel

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 0) {
                    ForEach(Array(viewModel.entries.enumerated()), id: \.offset) { index, entry in
                        Text(entry)
                            .font(.system(size: 11, design: .monospaced))
                            .textSelection(.enabled)
                            .id(index)
                    }
                }
                .padding(8)
            }
            .onChange(of: viewModel.entries.count) {
                if let lastIndex = viewModel.entries.indices.last {
                    proxy.scrollTo(lastIndex, anchor: .bottom)
                }
            }
        }
        .background(.black.opacity(0.4))
    }
}
