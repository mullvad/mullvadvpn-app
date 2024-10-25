//
//  CustomDNSViewModel.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-10-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import SwiftUI
import Combine

class CustomDNSViewModel: ObservableObject {
    @Published var blockAll = false
    @Published var blockAdvertising = false
    @Published var blockTracking = false
    @Published var blockMalware = false
    @Published var blockAdultContent = false
    @Published var blockGambling = false
    @Published var blockSocialMedia = false

    @Published var isEditing = false
    @Published var isExpanded = false
    var bucket: AnyCancellable?

    var headerTitle: String {
        "DNS content blockers \(enabledSettingsCount())"
    }

    func enabledSettingsCount() -> String {
        let enabledSettingsCount = selfFields.filter { $0 == true }.count
        return enabledSettingsCount == 0 ? "" : "(\(enabledSettingsCount))"
    }

    lazy var selfFields: [Bool] = [
        blockAdvertising,
        blockTracking,
        blockMalware,
        blockAdultContent,
        blockGambling,
        blockSocialMedia,
    ]

    lazy var contentBlockers: [DNSRowViewModel] = [
        DNSRowViewModel(name: "All", isEnabled: blockAll, action: toggleAll),
        DNSRowViewModel(name: DNSBlockingOptions.blockAdvertising.name, isEnabled: blockAdvertising),
        DNSRowViewModel(name: DNSBlockingOptions.blockTracking.name, isEnabled: blockTracking),
        DNSRowViewModel(
            name: DNSBlockingOptions.blockMalware.name,
            isEnabled: blockMalware,
            action: showMalwareInformation
        ),
        DNSRowViewModel(name: DNSBlockingOptions.blockAdultContent.name, isEnabled: blockAdultContent),
        DNSRowViewModel(name: DNSBlockingOptions.blockGambling.name, isEnabled: blockGambling),
        DNSRowViewModel(name: DNSBlockingOptions.blockSocialMedia.name, isEnabled: blockSocialMedia),
    ]

    func toggleAll() {
        blockAll.toggle()
        contentBlockers.forEach { $0.isEnabled = blockAll }
        contentBlockers.forEach { print($0) }
    }

    func showMalwareInformation() {
        print("show a popup here")
    }

    init(
        blockAdvertising: Bool = false,
        blockTracking: Bool = false,
        blockMalware: Bool = false,
        blockAdultContent: Bool = false,
        blockGambling: Bool = false,
        blockSocialMedia: Bool = false
    ) {
        self.blockAll = false
        self.blockAdvertising = blockAdvertising
        self.blockTracking = blockTracking
        self.blockMalware = blockMalware
        self.blockAdultContent = blockAdultContent
        self.blockGambling = blockGambling
        self.blockSocialMedia = blockSocialMedia

        contentBlockers.forEach { print($0) }

        bucket = $blockAll.sink { newvalue in
            self.contentBlockers.forEach { $0.isEnabled = newvalue }
        }
    }
}

extension DNSSettings {
    func viewModel() -> CustomDNSViewModel {
        CustomDNSViewModel(
            blockAdvertising: blockingOptions.contains(.blockAdvertising),
            blockTracking: blockingOptions.contains(.blockTracking),
            blockMalware: blockingOptions.contains(.blockMalware),
            blockAdultContent: blockingOptions.contains(.blockAdultContent),
            blockGambling: blockingOptions.contains(.blockGambling),
            blockSocialMedia: blockingOptions.contains(.blockSocialMedia)
        )
    }
}

class DNSRowViewModel: ObservableObject, Identifiable, CustomDebugStringConvertible {
    let name: String
    @Published var isEnabled: Bool
    let infoButtonAction: (() -> Void)?

    init(name: String, isEnabled: Bool, action: (() -> Void)? = nil) {
        self.name = name
        self.isEnabled = isEnabled
        self.infoButtonAction = action
    }

    var debugDescription: String {
        "\(Unmanaged.passUnretained(self).toOpaque()), name: \(name) isEnabled: \(isEnabled)"
    }
}
