//
//  CustomDNSSwiftUIView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-10-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI

struct CustomDNSSwiftUIView: View {
    @ObservedObject var viewModel: CustomDNSViewModel

    var body: some View {
        GeometryReader { _ in
            ScrollView {
                Section {
                    DisclosureGroup(viewModel.headerTitle, isExpanded: $viewModel.isExpanded) {
                        ForEach(viewModel.contentBlockers) { singleSetting in
                            DNSItemRow(viewModel: singleSetting)
                                .background(Color.Cell.Background.normal)
                        }
                    }
                    .listRowBackground(Color.Cell.Background.normal)
                    .foregroundColor(.white)
                }
                .background(Color.Cell.Background.normal)
                .foregroundColor(.white)
            }
        }
        .background(Color.secondaryColor)
    }
}

struct DNSItemRow: View {
    @ObservedObject var viewModel: DNSRowViewModel

    var body: some View {
        HStack {
            Spacer()
            Toggle(isOn: $viewModel.isEnabled, label: {
                Text(viewModel.name)
            }).toggleStyle(RedToggleStyle(action: viewModel.infoButtonAction))
        }
    }
}

#Preview {
    CustomDNSSwiftUIView(viewModel: CustomDNSViewModel())
}
