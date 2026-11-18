// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import SwiftUI

struct ConfigureApiAccessMethodView: View {
    @State var viewModel: any ConfigureApiAccessMethodViewModelProtocol
    @Environment(\.dismiss) private var dismiss
    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: any ConfigureApiAccessMethodViewModelProtocol) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                nameField
                typeField
                //ServerDetailsSection(viewModel: viewModel)
            }
        }
        .scrollBounceBehavior(.automatic)
        .padding(16)
        .background(Color.mullvadBackground)
        .safeAreaInset(edge: .top, spacing: 8.0) {
            VStack(spacing: 0) {
                // Prompt
                Text("The app will test the method before saving.")
                    .font(.mullvadTiny)
                    .frame(maxWidth: .infinity)
                    .foregroundStyle(Color.mullvadTextPrimary)
                    .padding(.vertical, 16)

                // Navigation row
                ZStack {
                    Text(viewModel.title)
                        .font(.mullvadSmall)
                        .multilineTextAlignment(.center)
                        .foregroundStyle(Color.mullvadTextPrimary)
                        .frame(maxWidth: .infinity)

                    HStack {
                        Button(action: {
                            dismiss()
                        }) {
                            HStack {
                                if let image = viewModel.dismissButtonImage {
                                    image
                                }
                                Text(viewModel.dismissButtonTitle)
                                    .font(.mullvadSmall)
                            }
                        }

                        Spacer()

                        Button(viewModel.saveButtonTitle) {
                            // Save action
                        }
                        .font(.mullvadSmall)
                    }
                    .foregroundStyle(Color.secondaryTextColor)
                }
                .padding(16.0)
            }
        }
        .accessibilityIdentifier(.addAccessMethodTableView)
        .toolbar(.hidden, for: .navigationBar)
        .tint(Color.mullvadTextPrimary)
        .toolbarBackground(Color.mullvadBackground, for: .navigationBar)
        .toolbarBackground(.visible, for: .navigationBar)
    }

    private var nameField: some View {
        SegmentedListItem(
            isLastInList: true,
            userInteraction: .enabledWithoutHighlight,
            leading: {
                itemFactory.leading(
                    for: .generic(title: NSLocalizedString("Name", comment: ""))
                )
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Required"),
                    text: $viewModel.name,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear)
                )
            }
        )
        .accessibilityIdentifier(
            AccessibilityIdentifier.accessMethodNameTextField
        )
        .padding(.bottom, 24)
    }

    private var typeField: some View {
        SegmentedListItem(
            isLastInList: true,
            userInteraction: .enabledWithoutHighlight,
            leadingAndTrailingDestination: {
                AccessMethodTypeView(
                    viewModel: AccessMethodTypeViewModel(
                        current: $viewModel.type, list: AccessMethodKind.allUserDefinedKinds)
                )
                .navigationTitle("Type")
                .toolbarTitleDisplayMode(.inline)
            },
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Type", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .drillDown(title: viewModel.type.localizedDescription, breadcrumb: nil))

            }
        )
        .accessibilityIdentifier(AccessibilityIdentifier.accessMethodProtocolPickerView)
        .padding(.bottom, 24.0)
    }
}

#Preview {
    ConfigureApiAccessMethodView(viewModel: ConfigureApiAccessMethodViewModel(accessMethod: nil))
}
