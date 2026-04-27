//
//  SpeedConnectionView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SpeedConnectionView<ViewModel: SpeedConnectionViewModelProtocol>: View {
    @ObservedObject var viewModel: ViewModel
    
    init(viewModel: ViewModel) {
        self.viewModel = viewModel
    }
    
    var body: some View {
        VStack(alignment: .leading,spacing: 8.0) {
            let bitsToKB = 1024.0
            Text(viewModel.downloadText)
                .font(.mullvadTiny)
                .foregroundStyle(Color.mullvadTextSecondary)
            
            Text(viewModel.uploadText)
                .font(.mullvadTiny)
                .foregroundStyle(Color.mullvadTextSecondary)
        }
        .padding(8.0)
        .onAppear {
            viewModel.startMonitoring()
        }
        .onDisappear {
            viewModel.stopMonitoring()
        }
    }
}
