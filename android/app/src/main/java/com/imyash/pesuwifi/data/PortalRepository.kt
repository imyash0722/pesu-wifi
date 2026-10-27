package com.imyash.pesuwifi.data

import android.content.Context
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.os.SystemClock
import com.imyash.pesuwifi.util.AppLogger
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.withContext
import java.net.Inet4Address
import java.net.InetAddress

data class PortalStatus(
    val isWifiConnected: Boolean = false,
    val isPesuWifi: Boolean = false,
    val isPortalOnline: Boolean = false,
    val isLoggedIn: Boolean = false,
    val activeUsername: String? = null,
    val latencyMs: Long? = null,
    val lastCheckedTimestamp: Long = 0,
    val statusMessage: String = "Initializing...",
    val isVpnActive: Boolean = false
)

class PortalRepository(
    private val context: Context,
    private val accountRepository: AccountRepository = AccountRepository.getInstance(context),
    private val api: PortalApi = PortalApi
) {
    private val connectivityManager =
        context.getSystemService(Context.CONNECTIVITY_SERVICE) as? ConnectivityManager

    private val _statusFlow = MutableStateFlow(PortalStatus())
    val statusFlow: StateFlow<PortalStatus> = _statusFlow.asStateFlow()

    private var consecutiveProbeFailures = 0

    @Volatile
    private var lastConfirmedCampusNetwork: Network? = null
    @Volatile
    private var lastConfirmedCampusSsid: String? = null
    @Volatile
    private var lastConfirmedCampusTime: Long = 0L

    @Volatile
    private var isExplicitLogout: Boolean = false

    fun isExplicitlyLoggedOut(): Boolean = isExplicitLogout

    fun clearExplicitLogout() {
        isExplicitLogout = false
    }

    fun getWifiLocalAddress(wifiNet: Network? = null): InetAddress? {
        val net = wifiNet ?: getWifiNetwork() ?: return null
        val caps = connectivityManager?.getNetworkCapabilities(net) ?: return null
        if (caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)) return null

        val lp = connectivityManager?.getLinkProperties(net) ?: return null
        val iface = lp.interfaceName ?: ""
        if (iface.startsWith("tun") || iface.startsWith("wg") || iface.startsWith("ppp")) {
            return null
        }

        return lp.linkAddresses.map { it.address }
            .firstOrNull { it is Inet4Address && !it.isLoopbackAddress && !it.isLinkLocalAddress }
            ?.takeIf { addr ->
                // Guard against CGNAT VPN addresses (100.64.0.0/10) such as Tailscale
                val octets = addr.address.map { it.toInt() and 0xFF }
                !(octets.size == 4 && octets[0] == 100 && octets[1] in 64..127)
            }
    }

    /**
     * Resolves the active physical Wi-Fi Network instance across all available network interfaces.
     * Strictly filters out VPN networks that also declare TRANSPORT_WIFI as their underlying transport.
     */
    fun getWifiNetwork(): Network? {
        val cm = connectivityManager ?: return null
        return cm.allNetworks.firstOrNull { network ->
            val caps = cm.getNetworkCapabilities(network) ?: return@firstOrNull false
            val lp = cm.getLinkProperties(network)
            val iface = lp?.interfaceName ?: ""
            caps.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) &&
                !caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN) &&
                !iface.startsWith("tun") &&
                !iface.startsWith("wg") &&
                !iface.startsWith("ppp")
        }
    }

    fun isWifiConnected(): Boolean {
        return getWifiNetwork() != null
    }

    fun getCurrentWifiSsid(wifiNet: Network? = null): String? {
        val cm = connectivityManager ?: return null
        val targetNet = wifiNet ?: getWifiNetwork() ?: return null
        val caps = cm.getNetworkCapabilities(targetNet)
        val wifiInfo = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.Q) {
            caps?.transportInfo as? android.net.wifi.WifiInfo
        } else {
            @Suppress("DEPRECATION")
            val wm = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? android.net.wifi.WifiManager
            wm?.connectionInfo
        }
        val ssid = wifiInfo?.ssid?.takeIf { it != "<unknown ssid>" }
            ?: run {
                @Suppress("DEPRECATION")
                val wm = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? android.net.wifi.WifiManager
                wm?.connectionInfo?.ssid?.takeIf { it != "<unknown ssid>" }
            }
        val clean = ssid?.replace("\"", "")?.trim()
        return clean?.takeIf { it.isNotEmpty() && it != "<unknown ssid>" }
    }

    fun getCurrentWifiBssid(wifiNet: Network? = null): String? {
        val cm = connectivityManager ?: return null
        val targetNet = wifiNet ?: getWifiNetwork() ?: return null
        val caps = cm.getNetworkCapabilities(targetNet)
        val wifiInfo = if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.Q) {
            caps?.transportInfo as? android.net.wifi.WifiInfo
        } else {
            @Suppress("DEPRECATION")
            val wm = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? android.net.wifi.WifiManager
            wm?.connectionInfo
        }
        val bssid = wifiInfo?.bssid?.takeIf { it != "02:00:00:00:00:00" }
            ?: run {
                @Suppress("DEPRECATION")
                val wm = context.applicationContext.getSystemService(Context.WIFI_SERVICE) as? android.net.wifi.WifiManager
                wm?.connectionInfo?.bssid?.takeIf { it != "02:00:00:00:00:00" }
            }
        return bssid
    }

    fun isCampusSsid(ssid: String?): Boolean {
        if (ssid.isNullOrBlank()) return false
        val clean = ssid.replace("\"", "").trim()
        if (CAMPUS_SSIDS.any { it.equals(clean, ignoreCase = true) }) return true
        val upper = clean.uppercase()
        return upper.contains("PESU") ||
                upper.contains("PES-WIFI") ||
                upper.contains("PES_WIFI") ||
                upper.contains("PESUNIVERSITY") ||
                upper.contains("AMAATRA") ||
                upper.contains("FOODCOURT") ||
                upper.contains("PES SOUTH CAFE") ||
                upper.contains("PESU-CIE")
    }

    /**
     * Determines whether the given (or active) Wi-Fi network is the PESU campus network.
     * 1. If SSID is known, it is the authoritative source of truth (must contain "PESU").
     * 2. If SSID is redacted/unknown (e.g. during AP roaming handoffs or Android 12 location policy),
     *    check if this active connection was recently verified as campus Wi-Fi.
     * 3. If SSID is redacted and no recent cache, check campus-specific network infrastructure:
     *    Cyberoam gateway (192.168.254.*, 10.*), internal DNS (192.168.3.*), or campus IPs.
     */
    fun isCampusNetwork(wifiNet: Network? = null): Boolean {
        val net = wifiNet ?: getWifiNetwork() ?: return false

        // 1. Check SSID if available: if known, it is the definitive indicator
        val ssid = getCurrentWifiSsid(net)
        if (!ssid.isNullOrBlank()) {
            val isCampus = isCampusSsid(ssid)
            if (isCampus) {
                lastConfirmedCampusNetwork = net
                lastConfirmedCampusSsid = ssid
                lastConfirmedCampusTime = SystemClock.elapsedRealtime()
                return true
            } else {
                // Definitive non-campus SSID (home, personal hotspot, etc.)
                lastConfirmedCampusNetwork = null
                lastConfirmedCampusSsid = null
                lastConfirmedCampusTime = 0L
                return false
            }
        }

        // 1b. Check BSSID against campus AP database: 0ms recognition if router MAC is known
        val now = SystemClock.elapsedRealtime()
        val bssid = getCurrentWifiBssid(net)
        if (!bssid.isNullOrBlank() && BssidDatabase.isCampusBssid(context, bssid)) {
            lastConfirmedCampusNetwork = net
            if (ssid != null) lastConfirmedCampusSsid = ssid
            lastConfirmedCampusTime = now
            AppLogger.d("PortalRepository", "isCampusNetwork: BSSID $bssid matched campus AP database. Instant campus recognition.")
            return true
        }

        // 2. If SSID is redacted/unknown, check if recently confirmed on campus.
        // During roaming between campus APs, the SSID is momentarily null, but the device
        // remains continuously connected on campus.
        if (lastConfirmedCampusTime > 0L && (now - lastConfirmedCampusTime) < 120_000L) {
            if (lastConfirmedCampusNetwork == null || lastConfirmedCampusNetwork == net) {
                AppLogger.d("PortalRepository", "isCampusNetwork: SSID is null/redacted, but connection was confirmed campus ${(now - lastConfirmedCampusTime) / 1000}s ago (roaming handoff). Preserving campus status.")
                return true
            }
        }

        // 3. Only if SSID is redacted/unknown and no recent confirmation, check campus-specific network infrastructure
        val lp = connectivityManager?.getLinkProperties(net)
        if (lp != null) {
            val gw = lp.routes.firstOrNull { it.isDefaultRoute }?.gateway?.hostAddress
            val dns = lp.dnsServers.mapNotNull { it.hostAddress }
            val domains = lp.domains ?: ""
            val ips = lp.linkAddresses.mapNotNull { it.address?.hostAddress }

            val isCampusInfrastructure = (gw != null && (gw.startsWith("192.168.254.") || gw.startsWith("10.") || gw.startsWith("172.16."))) ||
                    dns.any { it.startsWith("192.168.3.") } ||
                    domains.contains("pesu", ignoreCase = true) ||
                    ips.any { it.startsWith("10.") || it.startsWith("172.16.") }

            if (isCampusInfrastructure) {
                lastConfirmedCampusNetwork = net
                lastConfirmedCampusTime = now
                AppLogger.d("PortalRepository", "isCampusNetwork: Infrastructure match (GW=$gw, DNS=$dns, Domains=$domains, IPs=$ips). Campus recognized.")
                return true
            }
        }

        return false
    }

    fun isVpnActive(): Boolean {
        val cm = connectivityManager ?: return false
        return cm.allNetworks.any { network ->
            val caps = cm.getNetworkCapabilities(network) ?: return@any false
            caps.hasTransport(NetworkCapabilities.TRANSPORT_VPN)
        }
    }

    fun getDefaultGateway(wifiNet: Network? = null): String? {
        val net = wifiNet ?: getWifiNetwork() ?: return null
        val lp = connectivityManager?.getLinkProperties(net) ?: return null
        return lp.routes.firstOrNull { it.isDefaultRoute && it.gateway != null }?.gateway?.hostAddress
            ?: lp.routes.firstOrNull { val gw = it.gateway; gw != null && !gw.isLoopbackAddress }?.gateway?.hostAddress
            ?: if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.R) lp.dhcpServerAddress?.hostAddress else null
    }

    fun resolvePortalBase(wifiNet: Network? = null): String {
        // 1. If dynamically detected via redirect or active candidate probe on this connection, reuse it
        PortalApi.detectedPortalBase?.let { return it }

        val ssid = getCurrentWifiSsid(wifiNet)
        val gw = getDefaultGateway(wifiNet)

        // 2. Explicit SSID-based matching (Highest Priority)
        if (ssid != null) {
            if (ssid.equals("AMAATRA-HOSTEL", ignoreCase = true) || ssid.contains("AMAATRA", ignoreCase = true)) {
                return if (gw == "172.16.1.1" || gw == "172.16.0.1" || gw == "192.168.1.1") "http://$gw:8090" else PortalApi.AMAATRA_PORTAL_BASE
            }
            if (ssid.contains("PESU", ignoreCase = true) ||
                ssid.equals("Foodcourt", ignoreCase = true) ||
                ssid.contains("cafe", ignoreCase = true) ||
                ssid.contains("CIE", ignoreCase = true)) {
                return PortalApi.DEFAULT_PORTAL_BASE
            }
        }

        // 3. DNS-based matching: if campus DNS (192.168.3.2) is present, it's EC/RR Campus
        val targetNet = wifiNet ?: getWifiNetwork()
        val dnsServers = targetNet?.let { connectivityManager?.getLinkProperties(it)?.dnsServers } ?: emptyList()
        if (dnsServers.any { it.hostAddress == "192.168.3.2" }) {
            return PortalApi.DEFAULT_PORTAL_BASE
        }

        // 4. Gateway-specific matching
        if (gw != null) {
            if (gw == "172.16.1.1" || gw == "172.16.0.1") {
                return PortalApi.AMAATRA_PORTAL_BASE
            }
            if (gw == "192.168.1.1") {
                return PortalApi.AMAATRA_LEGACY_BASE
            }
            // PESU EC/RR direct gateway or internal 10.* / 172.16.* VLAN routing -> Cyberoam is at 192.168.254.1:8090
            if (gw.startsWith("192.168.254.") || gw.startsWith("10.") || gw.startsWith("172.16.")) {
                return PortalApi.DEFAULT_PORTAL_BASE
            }
        }

        // 5. Default fallback: EC/RR campus Cyberoam gateway
        return PortalApi.DEFAULT_PORTAL_BASE
    }

    fun clearCampusCache() {
        lastConfirmedCampusNetwork = null
        lastConfirmedCampusSsid = null
        lastConfirmedCampusTime = 0L
        isExplicitLogout = false
        PortalApi.detectedPortalBase = null
    }

    suspend fun refreshStatus(): PortalStatus = withContext(Dispatchers.IO) {
        val wifiNet = getWifiNetwork()
        val cm = connectivityManager
        if (wifiNet != null) {
            try {
                cm?.bindProcessToNetwork(wifiNet)
            } catch (e: Exception) {
                AppLogger.d("PortalRepository", "bindProcessToNetwork failed: ${e.message}")
            }
        } else {
            try {
                cm?.bindProcessToNetwork(null)
            } catch (_: Exception) {}
        }

        val localIp = getWifiLocalAddress(wifiNet)
        val gw = getDefaultGateway(wifiNet)
        val vpnActive = isVpnActive()
        api.portalBaseUrl = resolvePortalBase(wifiNet)
        api.setWifiSocketFactory(wifiNet?.socketFactory, localIp)

        val activeUser = accountRepository.getActiveUser()

        if (wifiNet == null) {
            consecutiveProbeFailures = 0
            val status = PortalStatus(
                isWifiConnected = false,
                isPesuWifi = false,
                isPortalOnline = false,
                isLoggedIn = false,
                activeUsername = activeUser,
                lastCheckedTimestamp = System.currentTimeMillis(),
                statusMessage = "Wi-Fi disconnected",
                isVpnActive = vpnActive
            )
            _statusFlow.value = status
            AppLogger.d("PortalRepository", "refreshStatus: Wi-Fi is not connected")
            return@withContext status
        }

        val currentSsid = getCurrentWifiSsid(wifiNet)
        val isCampus = isCampusNetwork(wifiNet)

        val candidateGateways = listOfNotNull(
            gw?.takeIf { it == "172.16.1.1" || it == "172.16.0.1" || it == "192.168.254.1" || it == "192.168.1.1" }?.let { "http://$it:8090" }
        )
        val start = System.currentTimeMillis()
        val portalUp = api.isPortalOnline(candidateGateways)
        val latency = System.currentTimeMillis() - start

        if (portalUp) {
            lastConfirmedCampusNetwork = wifiNet
            if (!currentSsid.isNullOrBlank()) {
                lastConfirmedCampusSsid = currentSsid
            }
            lastConfirmedCampusTime = SystemClock.elapsedRealtime()
        }

        if (!portalUp) {
            if (isCampus) {
                // Connected to campus AP, but gateway probe failed (port 8090 blocked/slow or packet drop)
                // Check if device already has active internet access (e.g. NET_CAPABILITY_VALIDATED or generate_204)
                val caps = connectivityManager?.getNetworkCapabilities(wifiNet)
                val isValidated = caps?.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) == true
                val isCaptive = caps?.hasCapability(NetworkCapabilities.NET_CAPABILITY_CAPTIVE_PORTAL) == true

                val isInternetUp = !isExplicitLogout && ((isValidated && !isCaptive) || (api.verifyInternetConnectivity() == PortalApi.InternetProbeResult.ONLINE))
                if (isInternetUp) {
                    consecutiveProbeFailures = 0
                    val status = PortalStatus(
                        isWifiConnected = true,
                        isPesuWifi = true,
                        isPortalOnline = false,
                        isLoggedIn = true,
                        activeUsername = activeUser,
                        latencyMs = latency,
                        lastCheckedTimestamp = System.currentTimeMillis(),
                        statusMessage = "Connected as ${activeUser ?: "active session"}",
                        isVpnActive = vpnActive
                    )
                    _statusFlow.value = status
                    AppLogger.i("PortalRepository", "refreshStatus: Gateway port 8090 unresponsive, but internet connectivity is VALIDATED & ONLINE. Preserving loggedIn=true.")
                    return@withContext status
                }

                // Truly offline, captive portal, or explicitly logged out
                val status = PortalStatus(
                    isWifiConnected = true,
                    isPesuWifi = true, // Preserve campus classification so keepalive loop does not terminate
                    isPortalOnline = false,
                    isLoggedIn = false,
                    activeUsername = activeUser,
                    latencyMs = latency,
                    lastCheckedTimestamp = System.currentTimeMillis(),
                    statusMessage = if (isExplicitLogout) "PESU Wi-Fi connected (Logged out)" else "PESU Wi-Fi: Gateway probe unreachable (roaming or VPN active)",
                    isVpnActive = vpnActive
                )
                _statusFlow.value = status
                AppLogger.roam("PortalRepository", "refreshStatus: Campus network active (SSID='$currentSsid', IP='${localIp?.hostAddress}'), but gateway unreachable (explicitLogout=$isExplicitLogout, vpnActive=$vpnActive). Preserving campus mode (latency ${latency}ms)")
                return@withContext status
            } else {
                // Truly external Wi-Fi (Home, Hotspot, Office) where PESU gateway does not exist
                val status = PortalStatus(
                    isWifiConnected = true,
                    isPesuWifi = false,
                    isPortalOnline = false,
                    isLoggedIn = false,
                    activeUsername = activeUser,
                    latencyMs = latency,
                    lastCheckedTimestamp = System.currentTimeMillis(),
                    statusMessage = "External Wi-Fi (Keepalive paused)",
                    isVpnActive = vpnActive
                )
                _statusFlow.value = status
                AppLogger.i("PortalRepository", "refreshStatus: Connected to external Wi-Fi '$currentSsid' (gateway probe timed out)")
                return@withContext status
            }
        }

        val targetUser = activeUser ?: "test"
        var loggedIn = if (isExplicitLogout) false else api.checkLive(targetUser)

        if (loggedIn) {
            // Verify real internet routing to catch "Zombie Sessions" (Cyberoam says live, but AP intercepts)
            val probeResult = api.verifyInternetConnectivity()
            when (probeResult) {
                PortalApi.InternetProbeResult.CAPTIVE_PORTAL -> {
                    AppLogger.roam("PortalRepository", "ZOMBIE SESSION DETECTED: checkLive reported true, but captive portal intercepted HTTP traffic. Invalidating session state to force re-auth.")
                    loggedIn = false
                    consecutiveProbeFailures = 0
                }
                PortalApi.InternetProbeResult.ONLINE -> {
                    consecutiveProbeFailures = 0
                    try {
                        connectivityManager?.reportNetworkConnectivity(wifiNet, true)
                    } catch (e: Exception) {
                        // Ignore security or OEM restrictions
                    }
                }
                PortalApi.InternetProbeResult.FAILED -> {
                    // Do NOT invalidate loggedIn when probe times out or DNS has brief glitches!
                    // Cyberoam explicitly confirmed the session is active (ack="ack").
                    // Only an actual captive portal redirect (CAPTIVE_PORTAL) indicates a zombie session.
                    AppLogger.d("PortalRepository", "Internet probe timed out/unreachable, but Cyberoam confirmed session is live. Preserving loggedIn=true.")
                }
            }
        } else {
            // When checkLive is false (ack="nack"), device is NOT authenticated as targetUser.
            // When VPN is active, Android's NET_CAPABILITY_VALIDATED is a false positive
            // caused by the VPN tunnel, NOT genuine Wi-Fi portal authorization!
            val caps = connectivityManager?.getNetworkCapabilities(wifiNet)
            val isValidated = caps?.hasCapability(NetworkCapabilities.NET_CAPABILITY_VALIDATED) == true
            val isCaptive = caps?.hasCapability(NetworkCapabilities.NET_CAPABILITY_CAPTIVE_PORTAL) == true
            if (!vpnActive && !isExplicitLogout && isValidated && !isCaptive) {
                val directProbe = api.verifyInternetConnectivity()
                if (directProbe == PortalApi.InternetProbeResult.ONLINE) {
                    AppLogger.i("PortalRepository", "refreshStatus: checkLive returned false for '$targetUser', but direct Wi-Fi internet is validated. Preserving loggedIn=true.")
                    loggedIn = true
                }
            }
            consecutiveProbeFailures = 0
        }

        val message = if (loggedIn) {
            try {
                connectivityManager?.bindProcessToNetwork(null)
            } catch (_: Exception) {}
            "Connected as ${activeUser ?: "active session"}"
        } else {
            "PESU Wi-Fi connected (Logged out)"
        }

        val status = PortalStatus(
            isWifiConnected = true,
            isPesuWifi = true,
            isPortalOnline = true,
            isLoggedIn = loggedIn,
            activeUsername = activeUser,
            latencyMs = latency,
            lastCheckedTimestamp = System.currentTimeMillis(),
            statusMessage = message,
            isVpnActive = vpnActive
        )
        _statusFlow.value = status
        AppLogger.i("PortalRepository", "refreshStatus: PESU Wi-Fi active (loggedIn=$loggedIn, latency=${latency}ms)")
        status
    }

    private fun isFailoverCandidateError(e: Throwable?): Boolean {
        if (e is PortalLoginException && e.isFailoverCandidate) return true
        val msg = e?.message?.lowercase() ?: return false
        return msg.contains("limit") ||
            msg.contains("maximum") ||
            msg.contains("transfer limit") ||
            msg.contains("quota") ||
            msg.contains("expired") ||
            msg.contains("validity") ||
            msg.contains("disabled") ||
            msg.contains("locked") ||
            msg.contains("does not exist") ||
            msg.contains("could not log you on") ||
            msg.contains("password")
    }

    suspend fun login(targetUsername: String? = null): Result<String> = withContext(Dispatchers.IO) {
        isExplicitLogout = false
        val wifiNet = getWifiNetwork()
        if (wifiNet == null) {
            AppLogger.d("PortalRepository", "Login aborted: Wi-Fi is not connected")
            return@withContext Result.failure<String>(Exception("Wi-Fi is not connected"))
        }
        try {
            connectivityManager?.bindProcessToNetwork(wifiNet)
        } catch (_: Exception) {}
        api.portalBaseUrl = resolvePortalBase(wifiNet)
        api.setWifiSocketFactory(wifiNet.socketFactory, getWifiLocalAddress(wifiNet))

        val username = targetUsername ?: accountRepository.getActiveUser()
            ?: run {
                AppLogger.w("PortalRepository", "Login aborted: No account configured")
                return@withContext Result.failure<String>(Exception("No account configured. Add an account first."))
            }
        val password = accountRepository.getPassword(username)
            ?: run {
                AppLogger.w("PortalRepository", "Login aborted: No password saved for $username")
                return@withContext Result.failure<String>(Exception("No password saved for '$username'"))
            }
        val currentStatus = _statusFlow.value
        val currentUser = currentStatus.activeUsername
        if (currentStatus.isLoggedIn && !currentUser.isNullOrBlank() && currentUser != username) {
            AppLogger.i("PortalRepository", "Logging out current user '$currentUser' before switching to '$username'")
            api.logout(currentUser)
            kotlinx.coroutines.delay(300L)
        }

        AppLogger.i("PortalRepository", "Initiating login for $username")
        val result = api.login(username, password)
        if (result.isSuccess) {
            accountRepository.setActiveUser(username)
            wifiNet?.let { net ->
                try {
                    connectivityManager?.reportNetworkConnectivity(net, true)
                } catch (e: Exception) {
                    // Ignore security or OEM restrictions
                }
            }
            refreshStatus()
            return@withContext result
        }

        // Multi-Account Auto-Failover:
        // When active account encounters concurrent limit or quota exhaustion, automatically try backup accounts
        val errorException = result.exceptionOrNull()
        if (isFailoverCandidateError(errorException)) {
            val allAccounts = accountRepository.getAllAccounts()
            val backupUsers = allAccounts.keys.filter { it != username }
            if (backupUsers.isNotEmpty()) {
                val reason = errorException?.message ?: "Account limit reached"
                AppLogger.roam("PortalRepository", "Account '$username' hit limit ($reason). Attempting Multi-Account Auto-Failover across ${backupUsers.size} backup account(s)...")

                for (backupUser in backupUsers) {
                    val backupPass = allAccounts[backupUser] ?: continue
                    AppLogger.i("PortalRepository", ">>> Failover: Attempting sign-in with backup account '$backupUser'...")

                    // Evict potential stale session on backup account
                    api.logout(backupUser)
                    kotlinx.coroutines.delay(350L)

                    val backupResult = api.login(backupUser, backupPass)
                    if (backupResult.isSuccess) {
                        accountRepository.setActiveUser(backupUser)
                        wifiNet?.let { net ->
                            try {
                                connectivityManager?.reportNetworkConnectivity(net, true)
                            } catch (_: Exception) {}
                        }
                        refreshStatus()
                        val successNote = "Signed in as $backupUser (Auto-failover from $username: $reason)"
                        AppLogger.portal("PortalRepository", ">>> Multi-Account Auto-Failover SUCCESS: $successNote")
                        return@withContext Result.success(successNote)
                    } else {
                        AppLogger.w("PortalRepository", "Failover attempt for '$backupUser' failed: ${backupResult.exceptionOrNull()?.message}")
                    }
                }
            }
        }

        result
    }

    suspend fun logout(): Result<String> = withContext(Dispatchers.IO) {
        isExplicitLogout = true
        val wifiNet = getWifiNetwork()
        if (wifiNet == null) {
            AppLogger.d("PortalRepository", "Logout: Wi-Fi is not connected, clearing session locally")
            _statusFlow.value = _statusFlow.value.copy(
                isLoggedIn = false,
                statusMessage = "Wi-Fi disconnected (Logged out)"
            )
            return@withContext Result.success("Logged out locally (Wi-Fi disconnected)")
        }
        try {
            connectivityManager?.bindProcessToNetwork(wifiNet)
        } catch (_: Exception) {}
        api.portalBaseUrl = resolvePortalBase(wifiNet)
        api.setWifiSocketFactory(wifiNet.socketFactory, getWifiLocalAddress(wifiNet))

        // Immediately update statusFlow so UI switches to disconnected state without waiting
        val current = _statusFlow.value
        _statusFlow.value = current.copy(
            isLoggedIn = false,
            statusMessage = "PESU Wi-Fi connected (Logged out)"
        )

        wifiNet?.let { net ->
            try {
                connectivityManager?.reportNetworkConnectivity(net, false)
            } catch (_: Exception) {}
        }

        val username = accountRepository.getActiveUser() ?: "user"
        AppLogger.i("PortalRepository", "Initiating logout for $username")
        val result = api.logout(username)
        refreshStatus()
        result
    }

    companion object {
        val CAMPUS_SSIDS = setOf(
            "AMAATRA-HOSTEL",
            "AMAATRA_HOSTEL",
            "Foodcourt",
            "PESU-EC-Campus",
            "PESU-RR-Campus",
            "PESU-CIE",
            "pes south cafe",
            "PES-WIFI",
            "PES_WIFI",
            "pesuniversity-guest"
        )

        @Volatile
        private var instance: PortalRepository? = null

        fun getInstance(context: Context): PortalRepository {
            return instance ?: synchronized(this) {
                instance ?: PortalRepository(context.applicationContext).also { instance = it }
            }
        }
    }
}
