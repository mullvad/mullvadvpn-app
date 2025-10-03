struct LocationContext {
    var locations: [LocationNode]
    var customLists: [LocationNode]
    var filter: [SelectLocationFilter]
    var selectedLocation: LocationNode?
    var connectedRelayHostname: String?
    let selectLocation: (LocationNode) -> Void

    init(
        locations: [LocationNode],
        customLists: [LocationNode],
        filter: [SelectLocationFilter],
        selectedLocation: LocationNode?,
        connectedRelayHostname: String?,
        selectLocation: @escaping (LocationNode) -> Void
    ) {
        self.locations = locations
        self.customLists = customLists
        self.filter = filter
        self.selectedLocation = selectedLocation
        self.connectedRelayHostname = connectedRelayHostname
        self.selectLocation = selectLocation
    }
}
