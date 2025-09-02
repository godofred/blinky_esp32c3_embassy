#![allow(unexpected_cfgs)]

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::ipv4;
use esp_idf_svc::netif;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{self, AsyncWifi};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    wifi_dhcp_hostname: &'static str,
}

pub async fn async_connect_wifi<'d>(modem: Modem) -> Result<AsyncWifi<wifi::EspWifi<'d>>, EspError> {
    let wifi_config = CONFIG;
    let event_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let timer_service = esp_idf_svc::timer::EspTaskTimerService::new().unwrap();

    let netif_config = netif::NetifConfiguration {
        ip_configuration: Some(ipv4::Configuration::Client(
            ipv4::ClientConfiguration::DHCP(ipv4::DHCPClientSettings {
                hostname: Some(wifi_config.wifi_dhcp_hostname.try_into().unwrap()),
            }),
        )),
        ..netif::NetifConfiguration::wifi_default_client()
    };
    let dhcp_netif = netif::EspNetif::new_with_conf(&netif_config).expect("Could not ");

    let blocking_wifi_driver = wifi::WifiDriver::new(modem, event_loop.clone(), Some(nvs)).unwrap();
    let blocking_wifi = wifi::EspWifi::wrap_all(
        blocking_wifi_driver,
        dhcp_netif,
        #[cfg(esp_idf_esp_wifi_softap_support)]
        netif::EspNetif::new(esp_idf_svc::netif::NetifStack::Ap).unwrap(),
    )
    .unwrap();

    let mut async_wifi =
        wifi::AsyncWifi::wrap(blocking_wifi, event_loop.clone(), timer_service).unwrap();

    let client_config = wifi::ClientConfiguration {
        ssid: wifi_config.wifi_ssid.try_into().unwrap(),
        password: wifi_config.wifi_psk.try_into().unwrap(),
        ..Default::default()
    };

    let wifi_config = wifi::Configuration::Client(client_config);
    async_wifi.set_configuration(&wifi_config).unwrap();

    log::info!("Starting wifi...");
    async_wifi.start().await.unwrap();

    log::info!("Connecting wifi...");
    async_wifi.connect().await.unwrap();

    log::info!("Waiting for DHCP lease...");
    async_wifi.wait_netif_up().await.unwrap();

    let ip_info = async_wifi.wifi().sta_netif().get_ip_info().unwrap();
    let hostname = async_wifi.wifi().sta_netif().get_hostname().unwrap();
    log::info!("Device WiFi connection info: {:?}", ip_info);
    log::info!("Device DHCP hostname: {:?}", hostname);

    Ok(async_wifi)
}

pub fn sync_connect_wifi<'d>(modem: Modem) -> Result<wifi::EspWifi<'d>, EspError> {
    let wifi_config = CONFIG;
    let event_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let netif_config = netif::NetifConfiguration {
        ip_configuration: Some(ipv4::Configuration::Client(
            ipv4::ClientConfiguration::DHCP(ipv4::DHCPClientSettings {
                hostname: Some(wifi_config.wifi_dhcp_hostname.try_into().unwrap()),
            }),
        )),
        ..netif::NetifConfiguration::wifi_default_client()
    };
    let dhcp_netif = netif::EspNetif::new_with_conf(&netif_config).expect("Could not ");

    let blocking_wifi_driver = wifi::WifiDriver::new(modem, event_loop.clone(), Some(nvs)).unwrap();
    let mut blocking_wifi = wifi::EspWifi::wrap_all(
        blocking_wifi_driver,
        dhcp_netif,
        #[cfg(esp_idf_esp_wifi_softap_support)]
        netif::EspNetif::new(esp_idf_svc::netif::NetifStack::Ap).unwrap(),
    )
    .unwrap();

    let client_config = wifi::ClientConfiguration {
        ssid: wifi_config.wifi_ssid.try_into().unwrap(),
        password: wifi_config.wifi_psk.try_into().unwrap(),
        ..Default::default()
    };

    let wifi_config = wifi::Configuration::Client(client_config);
    blocking_wifi.set_configuration(&wifi_config).unwrap();

    log::info!("Starting wifi...");
    blocking_wifi.start().unwrap();

    log::info!("Connecting wifi...");
    blocking_wifi.connect().unwrap();

    let ip_info = blocking_wifi.sta_netif().get_ip_info().unwrap();
    let hostname = blocking_wifi.sta_netif().get_hostname().unwrap();
    log::info!("Device WiFi connection info: {:?}", ip_info);
    log::info!("Device DHCP hostname: {:?}", hostname);

    Ok(blocking_wifi)
}
