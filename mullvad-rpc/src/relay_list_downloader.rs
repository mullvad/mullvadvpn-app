/// Intended to be used to pre-load a relay list when creating an installer for the Mullvad VPN
/// app.

fn main() {
    let mut core = tokio_core::reactor::Core::new().expect("Failed to load old tokio runtime");
    let runtime = crate::MullvadRpcRuntime::new("dist-assets/api_root_ca.pem")
        .expect("Failed to load runtime");

    let relay_list_request =
        crate::RelayListProxy::new(runtime.mullvad_rest_handle()).relay_list_v3();

    let relay_list = core
        .run(relay_list_request)
        .expect("Failed to fetch relay list");

    println!("{}", serde_json::to_string_pretty(&relay_list).unwrap());

    let total_num_relays = relay_list
        .countries
        .map(|country| cities.map(|city| city.relays.len()).sum()).sum();

    let num_cities = relay_list.countries.map(|country| country.cities.len()).sum();

    let average_number_of_relays: f64 = total_num_relays / num_cities;



    println!("{}" average_number_of_relays);
}
