//
//  CustomDNSSwiftUIView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-10-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

class CustomDNSViewModel: ObservableObject {
    @Published var blockAll = false
    @Published var blockAdvertising = false
    @Published var blockTracking = false
    @Published var blockMalware = false
    @Published var blockAdultContent = false
    @Published var blockGambling = false
    @Published var blockSocialMedia = false
    @Published var enableCustomDNS = false
    @Published var customDNSDomains: [DNSServerEntry] = []

    @Published var isEditing = false
    @Published var isExpanded = false

    lazy var contentBlockers: [DNSRowViewModel] = [
        DNSRowViewModel(name: "All", isEnabled: blockAll),
        DNSRowViewModel(name: DNSBlockingOptions.blockAdvertising.name, isEnabled: blockAdvertising),
        DNSRowViewModel(name: DNSBlockingOptions.blockTracking.name, isEnabled: blockTracking),
        DNSRowViewModel(name: DNSBlockingOptions.blockMalware.name, isEnabled: blockMalware),
        DNSRowViewModel(name: DNSBlockingOptions.blockAdultContent.name, isEnabled: blockAdultContent),
        DNSRowViewModel(name: DNSBlockingOptions.blockGambling.name, isEnabled: blockGambling),
        DNSRowViewModel(name: DNSBlockingOptions.blockSocialMedia.name, isEnabled: blockSocialMedia),
    ]
}

class DNSRowViewModel: ObservableObject, Identifiable {
    let name: String
    @Published var isEnabled: Bool

    init(name: String, isEnabled: Bool) {
        self.name = name
        self.isEnabled = isEnabled
    }
}

struct CustomDNSSwiftUIView: View {
    @ObservedObject var viewModel: CustomDNSViewModel

    var body: some View {
        GeometryReader { _ in
            ScrollView {
                Section {
                    DisclosureGroup("DNS content blockers", isExpanded: $viewModel.isExpanded) {
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
            }).toggleStyle(RedToggleStyle())
        }
    }
}

#Preview {
    CustomDNSSwiftUIView(viewModel: CustomDNSViewModel())
}
