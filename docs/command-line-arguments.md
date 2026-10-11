# WiFi Connect Command Line Arguments

## Flags

*   **-h, --help**

    Prints help information

*   **-V, --version**

    Prints version information

## Options

Command line options have environment variable counterpart. If both a command line option and its environment variable counterpart are defined, the command line option will take higher precedence.

*   **-d, --portal-dhcp-range** dhcp_range, **$PORTAL_DHCP_RANGE**

    DHCP range of the captive portal WiFi network

    Default: _192.168.42.2,192.168.42.254_

*   **-g, --portal-gateway** gateway, **$PORTAL_GATEWAY**

    Gateway of the captive portal WiFi network

    Default: _192.168.42.1_

*   **-o, --portal-listening-port** listening_port, **$PORTAL_LISTENING_PORT**

    Listening port of the captive portal web server

    Default: _80_

*   **-i, --portal-interface** interface, **$PORTAL_INTERFACE**

    Wireless network interface to be used by WiFi Connect

*   **-p, --portal-passphrase** passphrase, **$PORTAL_PASSPHRASE**

    WPA2 Passphrase of the captive portal WiFi network

    Default: _no passphrase_

*   **-s, --portal-ssid** ssid, **$PORTAL_SSID**

    SSID of the captive portal WiFi network

    Default: _WiFi Connect_

*   **-a, --activity-timeout** timeout, **$ACTIVITY_TIMEOUT**

    Exit if no activity for the specified timeout (seconds)

    Default: _0 - no timeout_

*   **--activation-timeout** timeout, **$ACTIVATION_TIMEOUT**

    Wait up to this many seconds in total for a network submitted through the
    portal to connect. NetworkManager's own retries during that time count, and
    the wait ends early if NetworkManager rejects the credentials.

    Default: _0 - only the NetworkManager call timeout (15s)_

*   **--keep-failed-profile**, **$KEEP_FAILED_PROFILE**

    Keep the NetworkManager profile of a network that failed to connect instead
    of deleting it, so NetworkManager keeps retrying it after the portal exits.
    `$KEEP_FAILED_PROFILE` accepts `1` or `true`.

    Default: _profile is deleted_

*   **-u, --ui-directory** ui_directory, **$UI_DIRECTORY**

    Web UI directory location

    Default: _ui_
