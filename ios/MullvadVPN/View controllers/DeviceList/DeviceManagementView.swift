import MullvadTypes
import SwiftUI

struct DeviceManagementView: View {
    enum Style {
        case tooManyDevices((Bool) -> Void)
        case deviceManagement

        var actionButtonTitle: String {
            switch self {
            case .deviceManagement:
                NSLocalizedString("REMOVE_BUTTON_TITLE", comment: "")
            case .tooManyDevices:
                NSLocalizedString("DEVICE_LIMIT_LOGOUT_CONFIRMATION_BUTTON_TITLE", comment: "")
            }
        }

        var actionButtonStyle: MainButtonStyle.Style {
            switch self {
            case .tooManyDevices:
                .danger
            case .deviceManagement:
                .default
            }
        }

        func warningMessage(deviceName: String) -> String {
            return switch self {
            case .tooManyDevices:
                String(
                    format: NSLocalizedString(
                        "TOO_MANY_DEVICES_LOGOUT_CONFIRMATION",
                        value: "Are you sure you want to log **%@** out?",
                        comment: ""
                    ), deviceName.capitalized
                )

            case .deviceManagement:
                String(
                    format: NSLocalizedString(
                        "DEVICE_MANAGEMENT_REMOVE_CONFIRMATION",
                        value: """
                        Remove **%@**?
                        The device will be removed from the list and logged out.
                        """,
                        comment: ""
                    ), deviceName.capitalized
                )
            }
        }
    }

    let deviceManaging: any DeviceManaging
    let style: Style
    let onError: (String, Error) -> Void

    @State private var loggedInDevices: [DeviceListView.Device]?
    @State private var loading = true

    var canLoginNewDevice: Bool {
        guard let loggedInDevices else {
            return false
        }
        return loggedInDevices.count < ApplicationConfiguration.maxAllowedDevices
    }

    var bodyText: String {
        return switch style {
        case .deviceManagement:
            NSLocalizedString("DEVICE_MANAGEMENT_BODY_TEXT", value: """
            View and manage all your logged in devices. \
            You can have up to 5 devices on one account at a time. \
            Each device gets a name when logged in to help you tell them apart easily.
            """, comment: "")
        case .tooManyDevices:
            if canLoginNewDevice {
                NSLocalizedString("TOO_MANY_DEVICES_CAN_LOGIN_BODY_TEXT", value: """
                You can now continue logging in on this device.
                """, comment: "")
            } else {
                NSLocalizedString("TOO_MANY_DEVICES_LOGOUT_REQUIRED_BODY_TEXT", value: """
                Please log out of at least one by removing it from the list below. \
                You can find the corresponding device name under the device’s Account settings.
                """, comment: "")
            }
        }
    }

    private func fetchDevices() {
        loading = true
        _ = deviceManaging.getDevices { result in
            Task { @MainActor in
                loading = false
                switch result {
                case let .success(devices):
                    self.loggedInDevices = devices.map {
                        DeviceListView.Device(
                            id: $0.id,
                            name: $0.name.capitalized,
                            created: $0.created,
                            isCurrentDevice: $0.id == self.deviceManaging.currentDeviceId,
                            isBeingRemoved: false
                        )
                    }
                case let .failure(error):
                    onError(
                        NSLocalizedString(
                            "FAILED_TO_FETCH_DEVICES_TITLE",
                            value: "Failed to fetch devices",
                            comment: ""
                        ),
                        error
                    )
                }
            }
        }
    }

    @State var deviceManagementAlert: MullvadAlert?
    var body: some View {
        VStack {
            DeviceListView(
                devices: $loggedInDevices,
                loading: $loading,
                onRemoveDevice: { device in
                    deviceManagementAlert = MullvadAlert(
                        type: .warning,
                        message: style.warningMessage(deviceName: device.name),
                        action: .init(
                            type: style.actionButtonStyle,
                            title: style.actionButtonTitle,
                            identifier: AccessibilityIdentifier.logOutDeviceConfirmButton,
                            handler: {
                                await withCheckedContinuation { continuation in
                                    guard let loggedInDevices else {
                                        return
                                    }
                                    self.loggedInDevices = loggedInDevices.map {
                                        $0.id == device.id ? $0.setIsBeingRemoved(true) : $0
                                    }
                                    deviceManagementAlert = nil
                                    _ = deviceManaging.deleteDevice(
                                        device.id,
                                        completionHandler: { result in
                                            Task { @MainActor in
                                                switch result {
                                                case .success:
                                                    self.loggedInDevices?.removeAll(where: { $0.id == device.id })
                                                case let .failure(error):
                                                    self.loggedInDevices = loggedInDevices.map {
                                                        $0.id == device
                                                            .id ? $0.setIsBeingRemoved(false) : $0
                                                    }
                                                    onError(
                                                        NSLocalizedString(
                                                            "FAILED_TO_LOG_OUT_DEVICE_TITLE",
                                                            value: "Failed to log out device",
                                                            comment: ""
                                                        ),
                                                        error
                                                    )
                                                }
                                                continuation.resume()
                                            }
                                        }
                                    )
                                }
                            }
                        ),
                        dismissButtonTitle: NSLocalizedString("CANCEL_TITLE_BUTTON", value: "Cancel", comment: "")
                    )
                }, header: {
                    AnyView(VStack(alignment: .leading, spacing: 8) {
                        if case .tooManyDevices = style {
                            if canLoginNewDevice {
                                HStack {
                                    Spacer()
                                    Image.mullvadIconSuccess
                                    Spacer()
                                }
                                Text(NSLocalizedString(
                                    "DEVICE_DELETED_SUCCESSFULLY_MESSAGE",
                                    value: "Super!",
                                    comment: ""
                                ))
                                .font(.mullvadBig)
                                .foregroundStyle(Color.mullvadTextPrimary)
                            } else {
                                HStack {
                                    Spacer()
                                    Image.mullvadIconFail
                                    Spacer()
                                }
                                Text(NSLocalizedString(
                                    "TOO_MANY_DEVICES_TITLE",
                                    value: "Too many devices",
                                    comment: ""
                                ))
                                .font(.mullvadBig)
                                .foregroundStyle(Color.mullvadTextPrimary)
                            }
                        }
                        Text(bodyText)
                            .foregroundColor(.mullvadTextPrimary)
                            .opacity(0.6)
                            .font(.mullvadTinySemiBold)
                            .padding(.bottom, 16.0)
                    }
                    )
                }
            )
            Spacer()
            if case let .tooManyDevices(backToLogin) = style {
                MainButton(
                    text: NSLocalizedString("CONTINUE_WITH_LOGIN", value: "Continue with login", comment: ""),
                    style: .success
                ) {
                    backToLogin(true)
                }
                .accessibilityIdentifier(AccessibilityIdentifier.continueWithLoginButton)
                .disabled(!canLoginNewDevice)
                .padding(
                    EdgeInsets(
                        top: UIMetrics.contentLayoutMargins.top,
                        leading: UIMetrics.contentLayoutMargins.leading,
                        bottom: UIMetrics.contentLayoutMargins.bottom,
                        trailing: UIMetrics.contentLayoutMargins.trailing
                    )
                )
            }
        }
        .mullvadAlert(item: $deviceManagementAlert)
        .background(Color.mullvadBackground)
        .task {
            fetchDevices()
        }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(
            .deviceManagementView
        )
    }
}

#Preview {
    Text("TOO_MANY_DEVICES_TITLE")
        .sheet(isPresented: .constant(true)) {
            DeviceManagementView(
                deviceManaging: MockDeviceManaging(),
                style: .tooManyDevices { _ in },
                onError: { _, _ in }
            )
        }
}

#Preview("Too many devices: Success") {
    Text("")
        .sheet(isPresented: .constant(true)) {
            DeviceManagementView(
                deviceManaging: MockDeviceManaging(
                    devicesToReturn: ApplicationConfiguration.maxAllowedDevices - 1
                ),
                style: .tooManyDevices { _ in },
                onError: { _, _ in }
            )
        }
}

#Preview("Device Management") {
    Text("")
        .sheet(isPresented: .constant(true)) {
            NavigationView {
                DeviceManagementView(
                    deviceManaging: MockDeviceManaging(),
                    style: .deviceManagement,
                    onError: { _, _ in }
                )
                .navigationTitle("Manage Devices")
            }
        }
}
