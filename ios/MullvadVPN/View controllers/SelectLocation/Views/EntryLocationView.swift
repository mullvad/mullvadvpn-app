import SwiftUI

struct EntryLocationView<ViewModel: SelectLocationViewModel>: View {
    @Bindable var viewModel: ViewModel
    @Binding var customListSheetType: CustomListSheetType?
    let onScrollVisibilityChange: (Bool) -> Void

    var body: some View {
        if viewModel.showMultihopInfo {
            MultihopWhenNeededInfoView(viewModel: viewModel)
        } else {
            ExitLocationView(
                viewModel: viewModel, context: $viewModel.entryContext,
                customListSheetType: $customListSheetType,
                onScrollVisibilityChange: onScrollVisibilityChange)
        }
    }
}
