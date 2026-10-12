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

*   **--idle-timeout** timeout, **$IDLE_TIMEOUT**

    Once the portal page has been opened, exit if no request from a client
    arrives for this many seconds. Before the page is opened,
    `--activity-timeout` applies as usual. No deadline runs while a submitted
    network is being connected, and the idle clock restarts if that fails.

    Default: _0 - never (the portal stays up once opened)_

*   **--status-file** path, **$STATUS_FILE**

    Write the portal's state to this file as JSON, rewritten atomically on
    every change. Times are seconds on the system uptime clock (first field of
    `/proc/uptime`).

    ```json
    {"version":1,"pid":1234,"state":"opened","exit_reason":null,
     "started_at":512.3,"opened_at":530.1,"last_activity_at":561.8,
     "activity_timeout":600,"idle_timeout":300,
     "deadline":861.8,"updated_at":561.8}
    ```

    `state` is `waiting`, `opened`, `connecting` or `exited`; `exit_reason`
    (when exited) is `connected`, `timeout`, `idle`, `signal` or `error`.
    `deadline` is when the portal will close, or `null` if it has no deadline
    (no timeouts, or a connection attempt in progress). A file left by a
    process that was killed still names its `pid`, so check it is running.

    Default: _none_

*   **-u, --ui-directory** ui_directory, **$UI_DIRECTORY**

    Web UI directory location

    Default: _ui_
