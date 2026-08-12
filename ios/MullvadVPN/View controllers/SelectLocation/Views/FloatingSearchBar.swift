import SwiftUI

struct FloatingSearchBar: View {
    @Binding var searchText: String
    @Binding var isExpanded: Bool
    var isFocused: FocusState<Bool>.Binding

    @Namespace private var animation

    private enum AnimationID {
        case searchIcon
        case searchBackground
    }

    var body: some View {
        HStack(spacing: 0) {
            if isExpanded {
                ConfigurableTextField(
                    placeholder: "Search location or server",
                    text: $searchText,
                    isFocused: isFocused,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(
                        cornerRadius: 28.0,
                        backgroundColor: Color.mullvadContainerBackground,
                        height: 48.0,
                        spacing: 0
                    ),
                    configuration: TextFieldNamespace.Configuration(
                        submitConfiguration: TextFieldNamespace.SubmitConfiguration(
                            label: .search,
                            action: {
                                if searchText.isEmpty {
                                    withAnimation {
                                        isExpanded = false
                                        isFocused.wrappedValue = false
                                    }
                                }
                            })),
                    leadingView: {
                        searchIcon
                            .padding(.leading, 8.0)
                    }
                )
                .accessibilityAddTraits(.isSearchField)
                .accessibilityIdentifier(.selectLocationSearchTextField)
                .matchedGeometryEffect(id: AnimationID.searchBackground, in: animation)

                Button {
                    searchText = ""
                    withAnimation {
                        isExpanded = false
                        isFocused.wrappedValue = false
                    }
                } label: {
                    Image.mullvadIconCross
                        .foregroundColor(.mullvadTextPrimary)
                        .frame(width: 48, height: 48)
                        .background(Color.mullvadContainerBackground)
                        .clipShape(Circle())
                }
                .accessibilityLabel(Text("Close search"))
                .accessibilityIdentifier(.closeSearchButton)
                .transition(.opacity)
            } else {
                Spacer()
                Button {
                    withAnimation {
                        isExpanded = true
                    }
                } label: {
                    searchIcon
                        .frame(width: 48, height: 48)
                        .background {
                            RoundedRectangle(cornerRadius: 28)
                                .fill(Color.mullvadContainerBackground)
                                .matchedGeometryEffect(id: AnimationID.searchBackground, in: animation)
                        }
                }
                .accessibilityLabel(Text("Search locations"))
                .accessibilityIdentifier(.selectLocationSearchTextField)
            }
        }
        .onChange(of: isExpanded) { _, expanded in
            if expanded {
                isFocused.wrappedValue = true
            }
        }
        // Prevents the keyboard safe area animation from compounding with the bar's
        // layout animation, which otherwise causes a visible bounce on the text field.
        .transformEffect(.identity)
    }

    private var searchIcon: some View {
        ResizableImageView(image: Image.mullvadIconSearch, dimension: .width(32.0))
            .matchedGeometryEffect(id: AnimationID.searchIcon, in: animation)
    }
}
