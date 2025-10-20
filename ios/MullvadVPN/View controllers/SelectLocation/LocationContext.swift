struct LocationContext {
    var locations: [LocationNode]
    var customLists: [LocationNode]
    var filter: [SelectLocationFilter]
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
        self.selectLocation = selectLocation
    }
}
