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
    @State private var savingAlert: MullvadAlert?
    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: any ConfigureApiAccessMethodViewModelProtocol) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                promptView
                nameField
                typeField
                serverDetailsSection
                testView
            }
        }
        .scrollBounceBehavior(.automatic)
        .padding(16)
        .background(Color.mullvadBackground)
        .onDisappear {
            viewModel.stopTesting()
        }
        .accessibilityIdentifier(.addAccessMethodTableView)
        .navigationBarTitleDisplayMode(.inline)
        .mullvadAlert(item: $savingAlert)
        .customNavigationTitle(viewModel.title)
        .onAppear {
            LocalNetworkProbe().triggerLocalNetworkPrivacyAlert()
        }
        .doneBarButton {
            viewModel.startTesting()
        }
    }

    private var promptView: some View {
        HStack(alignment: .center) {
            Spacer()
            ResizableImageView(image: .mullvadIconInfo, dimension: .width(18))
            Text("The app will test the method before saving.")
                .font(.mullvadSmallSemiBold)
                .multilineTextAlignment(.leading)
                .foregroundStyle(Color.mullvadTextSecondary)
            Spacer()
        }
        .padding(.bottom, 16)
    }

    private var nameField: some View {
        Group {
            SegmentedListItem(
                isLastInList: true,
                userInteraction: .enabledWithoutHighlight,
                accessibilityIdentifier: .accessMethodNameTextField,
                leading: {
                    AccessMethodLabelView(title: "Name", textColor: viewModel.borderStyle(for: .name).textColor)
                },
                trailing: {
                    ConfigurableTextField(
                        placeholder: LocalizedStringKey("Required"),
                        text: $viewModel.accessMethodViewModel.name,
                        borderStyle: .constant(.none),
                        appearance: InputViewAppearance(
                            backgroundColor: .clear,
                            showsClearButton: false
                        )
                    )
                }
            )
            ValidationErrorView(error: viewModel.hasError(for: .name)?.errorDescription)
        }
        .onChange(of: viewModel.accessMethodViewModel.name) {
            viewModel.clearErrors(for: [.name])
        }
    }

    private var typeField: some View {
        SegmentedListItem(
            isLastInList: true,
            userInteraction: .enabledWithoutHighlight,
            leadingAndTrailingDestination: {
                AccessMethodTypeView(
                    viewModel: AccessMethodTypeViewModel(
                        current: $viewModel.accessMethodViewModel.method, list: AccessMethodKind.allUserDefinedKinds)
                )
            },
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Type", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .drillDown(title: viewModel.accessMethodViewModel.method.localizedDescription, breadcrumb: nil)
                )

            }
        )
        .accessibilityIdentifier(AccessibilityIdentifier.accessMethodProtocolPickerView)
        .padding(.vertical, 24.0)
    }

    private var serverDetailsSection: some View {
        Group {
            if viewModel.accessMethodViewModel.method.hasProxyConfiguration {
                ServerDetailsSection(viewModel: viewModel)
            } else {
                EmptyView()
            }
        }
    }

    private var testView: some View {
        MethodTestingStatusView(
            status: viewModel.testState,
            onStop: {
                viewModel.stopTesting()
            }
        )
        .onChange(
            of: viewModel.testState,
            { _, newValue in
                guard case .unreachable = newValue else {
                    return
                }
                savingAlert = methodSavingAlert
            }
        )
        .padding(.vertical, 24.0)
    }

    private var methodSavingAlert: MullvadAlert {
        MullvadAlert(
            type: .info, messages: ["API unreachable, save method anyway?"],
            actions: [
                .init(
                    type: .secondary,
                    title: "Save",
                    identifier: AccessibilityIdentifier.accessMethodConfirmDeleteButton,
                    handler: {
                        savingAlert = nil
                        viewModel.save()
                        dismiss()
                    }),
                .init(
                    type: .primary,
                    title: "Cancel",
                    handler: {
                        savingAlert = nil
                    }
                ),
            ])
    }
}
