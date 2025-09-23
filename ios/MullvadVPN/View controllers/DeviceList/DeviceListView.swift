import SwiftUI

struct DeviceListView: View {
    @Binding var devices: [Device]?
    @Binding var loading: Bool
    var onRemoveDevice: ((Device) -> Void)?
    let header: (() -> AnyView)?

    struct Device: Identifiable, Hashable {
        let id: String
        let name: String
        let created: Date
        let isCurrentDevice: Bool
        var isBeingRemoved: Bool

        func setIsBeingRemoved(_ isBeingRemoved: Bool) -> Self {
            var updatedSelf = self
            updatedSelf.isBeingRemoved = isBeingRemoved
            return updatedSelf
        }
    }

    var body: some View {
        let headerContent: () -> some View = {
            VStack {
                if let header {
                    header()
                }

                if loading {
                    Spacer()

                    VStack(spacing: 16) {
                        ProgressView()
                            .progressViewStyle(MullvadProgressViewStyle())

                        Text("Fetching devices...")
                            .foregroundColor(.mullvadTextPrimary.opacity(0.6))
                    }

                    Spacer()
                }
            }
        }

        MullvadList(
            devices ?? [],
            header: headerContent,
            footer: { EmptyView() },
            content: { device in
                MullvadListActionItemView(
                    item: .init(
                        id: device.id,
                        title: LocalizedStringKey(device.name),
                        state: device.isCurrentDevice ? "Current device" : nil,
                        detail: "Created: \(device.created.formatted(date: .long, time: .omitted))",
                        accessibilityIdentifier: .deviceCellRemoveButton,
                        pressed: {
                            onRemoveDevice?(device)
                        }
                    ),
                    icon: {
                        if !device.isCurrentDevice {
                            if device.isBeingRemoved {
                                ProgressView()
                                    .progressViewStyle(MullvadProgressViewStyle())
                                    .frame(width: 24, height: 24)
                                    .accessibilityIdentifier(.deviceRemovalProgressView)
                            } else {
                                Image.mullvadIconClose
                            }
                        }
                    }
                )
            }
        )
        .accessibilityIdentifier(.deviceManagementView)
    }
}

#Preview {
    DeviceListView(
        devices: .constant([
            DeviceListView.Device(
                id: "1",
                name: "Test device",
                created: Date(),
                isCurrentDevice: false,
                isBeingRemoved: true
            ),
            DeviceListView.Device(
                id: "2",
                name: "Test device",
                created: Date(),
                isCurrentDevice: false,
                isBeingRemoved: false
            ),
        ]),
        loading: .constant(false),
        onRemoveDevice: nil,
        header: nil
    )
}

#Preview("Loading") {
    DeviceListView(
        devices: .constant([]),
        loading: .constant(true),
        onRemoveDevice: nil,
        header: nil
    )
    .background(Color.mullvadBackground)
}
