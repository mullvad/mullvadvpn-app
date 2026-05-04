//
//  ListAccessMethodViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import SwiftUI

protocol ListAccessViewModel: ObservableObject {
    var items: [ListAccessMethodItem] { get }
    var itemInUse: ListAccessMethodItem? { get }
    func addNewMethod()
    func methodSelected(_ method: ListAccessMethodItem)
    func showAbout()
    func cipherIsValid(for item: ListAccessMethodItem) -> Bool
}

struct ListAccessMethodView<ViewModel>: View where ViewModel: ListAccessViewModel {
    @ObservedObject var viewModel: ViewModel

    init(viewModel: ViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            let text = NSLocalizedString(
                "Manage and add custom methods to access the Mullvad API.",
                comment: ""
            )
            let about = NSLocalizedString("About API access…", comment: "")

            MullvadList(
                viewModel.items,
                header: {
                    MullvadInfoHeaderView(
                        bodyText: "\(text) ",
                        link: about,
                        onTapLink: viewModel.showAbout
                    )
                },
                footer: {
                    MainButton(
                        text: LocalizedStringKey("Add"),
                        style: .default
                    ) {
                        viewModel.addNewMethod()
                    }
                    .accessibilityIdentifier(AccessibilityIdentifier.addAccessMethodButton.asString)
                },
                content: { item in
                    let accessibilityId: AccessibilityIdentifier? =
                        switch item.id {
                        case AccessMethodRepository.directId:
                            AccessibilityIdentifier.accessMethodDirectCell
                        case AccessMethodRepository.bridgeId:
                            AccessibilityIdentifier.accessMethodBridgesCell
                        case AccessMethodRepository.encryptedDNSId:
                            AccessibilityIdentifier.accessMethodEncryptedDNSCell
                        case AccessMethodRepository.domainFrontingId:
                            AccessibilityIdentifier.accessMethodDomainFrontingCell
                        default:
                            nil
                        }

                    MullvadListNavigationItemView(
                        item: MullvadListNavigationItem(
                            id: item.id,
                            title: item.name,
                            state: getState(for: item),
                            detail: item.detail,
                            accessibilityIdentifier: accessibilityId
                        ) {
                            viewModel.methodSelected(item)
                        }
                    )
                }
            )
            .accessibilityIdentifier(
                AccessibilityIdentifier.apiAccessListView.asString
            )
            .scrollBounceBehavior(.basedOnSize)
            Spacer()
        }
        .background(Color.mullvadBackground)
    }

    func getState(for item: ListAccessMethodItem) -> MullvadListNavigationItem.State? {
        if viewModel.cipherIsValid(for: item) {
            viewModel.itemInUse?.id == item.id
                ? .inUse
                : (!item.isEnabled
                    ? .off
                    : nil)
        } else {
            .warning("Unsupported cipher")
        }
    }
}

#Preview {
    NavigationView {
        ListAccessMethodView(
            viewModel: ListAccessViewModelBridge(
                interactor: ListAccessMethodInteractor(
                    repository: AccessMethodRepository(
                        shadowsocksCiphers: []
                    )
                ),
                delegate: nil
            )
        )
        .navigationTitle("API Access")
    }
}
