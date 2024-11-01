use windows::Win32::System::Wmi::{
    IWbemClassObject, WBEM_E_NOT_FOUND, WBEM_FLAG_RETURN_WBEM_COMPLETE,
};
use windows_core::{BSTR, PCWSTR, VARIANT};

/// Name of the blocking Hyper-V rule.
const BLOCK_RULE_ELEMENT_NAME: &str = "Mullvad VPN block-all rule";

/// Unique instance ID identifying the blocking Hyper-V rule.
const BLOCK_RULE_UUID: &str = "{319400cb-0445-4c1b-a081-1cbc57cdbcb8}";

const WMI_NAMESPACE: &str = "root\\standardcimv2";

/// Errors occurring while configuring Hyper-V firewall rules
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to initialize the COM library")]
    InitializeCom(#[source] wmi::WMIError),
    #[error("Failed to connect to the WMI namespace '{WMI_NAMESPACE}'")]
    ConnectWmi(#[source] wmi::WMIError),
    #[error("Failed to obtain Hyper-V rule class")]
    ObtainHyperVClass(#[source] wmi::WMIError),
    #[error("Failed to create new instance of Hyper-V rule class")]
    NewRuleInstance(#[source] windows_core::Error),
    #[error("Failed to set rule setting: {0}")]
    SetRuleKey(&'static str, #[source] windows_core::Error),
    #[error("Failed to put the rule instance")]
    PutInstance(#[source] windows_core::Error),
    #[error("Failed to delete rule instance")]
    DeleteInstance(#[source] windows_core::Error),
}

/// Initialize WMI connection to the ROOT\StandardCIMV2 namespace, which may be used for
/// interacting with Hyper-V rules.
pub fn init_wmi() -> Result<wmi::WMIConnection, Error> {
    wmi::WMIConnection::with_namespace_path(
        WMI_NAMESPACE,
        wmi::COMLibrary::new().map_err(Error::InitializeCom)?,
    )
    .map_err(Error::ConnectWmi)
}

/// Add a Hyper-V rule that blocks all traffic using WMI (Windows Management Instrumentation).
///
/// Instances of the WMI class `MSFT_NetFirewallHyperVRule` in the namespace "root\standardcimv2"
/// belong to the same firewall ruleset as that visible in PowerShell using the command
/// `Get-NetFirewallHyperVRule`.
///
/// Details about the `MSFT_NetFirewallHyperVRule`, including the meaning of properties, are
/// documented here:
/// https://learn.microsoft.com/en-us/windows/win32/fwp/wmi/wfascimprov/msft-netfirewallhypervrule
///
/// `con` must be a valid WMI connection for the `root\standardcimv2` WMI namespace. Such a connection
/// can be initialized using [`init_wmi`].
pub fn add_blocking_hyperv_firewall_rule(con: &wmi::WMIConnection) -> Result<(), Error> {
    let class = con
        .get_raw_by_path("MSFT_NetFirewallHyperVRule")
        .map_err(Error::ObtainHyperVClass)?;

    // SAFETY: We have a valid class wrapper, so spawning instances is safe
    let instance = unsafe { class.inner.SpawnInstance(0) }.map_err(Error::NewRuleInstance)?;

    put_instance_property(
        &instance,
        "ElementName",
        &VARIANT::from(BSTR::from(BLOCK_RULE_ELEMENT_NAME)),
    )?;
    put_instance_property(
        &instance,
        "InstanceID",
        &VARIANT::from(BSTR::from(BLOCK_RULE_UUID)),
    )?;

    // Action: 4 = block
    put_instance_property(&instance, "Action", &VARIANT::from(4))?;

    // Enabled: 1 = enabled
    put_instance_property(&instance, "Enabled", &VARIANT::from(1))?;

    // Direction: 2 = outbound
    put_instance_property(&instance, "Direction", &VARIANT::from(2))?;

    // SAFETY: We have a valid instance
    unsafe {
        con.svc
            .PutInstance(&instance, WBEM_FLAG_RETURN_WBEM_COMPLETE, None, None)
            .map_err(Error::PutInstance)
    }
}

/// Set property for a WMI class instance `inst`.
fn put_instance_property(
    inst: &IWbemClassObject,
    prop: &'static str,
    val: &VARIANT,
) -> Result<(), Error> {
    let utf16_prop: Vec<_> = prop.encode_utf16().chain(std::iter::once(0u16)).collect();

    // SAFETY: All arguments are valid and properly null-terminated
    unsafe {
        inst.Put(PCWSTR(utf16_prop.as_ptr()), 0, val, 0)
            .map_err(|error| Error::SetRuleKey(prop, error))
    }
}

/// Remove Hyper-V rule previously added by [`add_blocking_hyperv_firewall_rule`]. See the
/// documentation of that function for more details.
///
/// This function succeeds if the rule is not present or has already been removed.
///
/// `con` must be a valid WMI connection for the `root\standardcimv2` WMI namespace. Such a connection
/// can be initialized using [`init_wmi`].
pub fn remove_blocking_hyperv_firewall_rule(con: &wmi::WMIConnection) -> Result<(), Error> {
    let rule_path = BSTR::from(format!(
        r#"MSFT_NetFirewallHyperVRule.InstanceID="{BLOCK_RULE_UUID}""#
    ));
    // SAFETY: All arguments are valid.
    unsafe {
        con.svc
            .DeleteInstance(&rule_path, WBEM_FLAG_RETURN_WBEM_COMPLETE, None, None)
            .or_else(map_deletion_err)
    }
}

fn map_deletion_err(err: windows_core::Error) -> Result<(), Error> {
    if err.code().0 == WBEM_E_NOT_FOUND.0 {
        // If the rule doesn't exist, do nothing
        Ok(())
    } else {
        Err(Error::DeleteInstance(err))
    }
}
