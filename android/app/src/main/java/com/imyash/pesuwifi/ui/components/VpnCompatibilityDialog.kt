package com.imyash.pesuwifi.ui.components

import android.content.Context
import android.content.Intent
import android.provider.Settings
import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.OpenInNew
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.imyash.pesuwifi.ui.theme.StatusGreen

data class VpnGuide(
    val id: String,
    val name: String,
    val shortName: String,
    val packageNames: List<String>,
    val description: String,
    val steps: List<String>,
    val lanNotice: String? = null
)

val VPN_GUIDES: List<VpnGuide> = listOf(
    VpnGuide(
        id = "universal",
        name = "Universal Guide (Any VPN App)",
        shortName = "Universal",
        packageNames = emptyList(),
        description = "Works on any Android VPN that supports Split Tunneling, App Bypassing, or Excluded Apps.",
        steps = listOf(
            "1. Open your VPN app and go to Settings (or Features / Connection).",
            "2. Find 'Split Tunneling', 'Bypasser', 'App Exclusions', or 'Allowed Apps'.",
            "3. Choose 'Bypass VPN' / 'Exclude Apps' and select 'PESU WiFi'."
        ),
        lanNotice = "If your VPN has a strict Killswitch ('Block connections without VPN'), enable 'Allow LAN / Local Network traffic' so local gateway probes can reach 192.168.254.1."
    ),
    VpnGuide(
        id = "tailscale",
        name = "Tailscale",
        shortName = "Tailscale",
        packageNames = listOf("com.tailscale.ipn"),
        description = "Exempt PESU WiFi from Tailscale while keeping all self-hosted nodes and remote access 100% active.",
        steps = listOf(
            "1. Open the Tailscale app.",
            "2. Tap Settings (gear icon) in the top bar.",
            "3. Tap 'App Split Tunnel'.",
            "4. Check / select 'PESU WiFi' from the app list."
        ),
        lanNotice = "If using a Tailscale Exit Node: tap your active Exit Node and ensure 'Allow LAN access' is checked."
    ),
    VpnGuide(
        id = "proton",
        name = "Proton VPN",
        shortName = "Proton",
        packageNames = listOf("ch.protonvpn.android"),
        description = "Exclude PESU WiFi so campus authentication succeeds without disconnecting Proton.",
        steps = listOf(
            "1. Open Proton VPN and tap Settings (gear icon).",
            "2. Tap 'Features' → 'Split Tunneling' and turn it ON.",
            "3. Select 'Exclude apps from VPN tunnel' and check 'PESU WiFi'."
        ),
        lanNotice = "Under Settings → Advanced, ensure 'Allow LAN connections' or 'Alternative Routing' is ON."
    ),
    VpnGuide(
        id = "wireguard",
        name = "WireGuard",
        shortName = "WireGuard",
        packageNames = listOf("com.wireguard.android"),
        description = "Exclude PESU WiFi from your WireGuard tunnel configuration.",
        steps = listOf(
            "1. Open WireGuard and tap your active tunnel.",
            "2. Tap the Edit (pencil) icon in the top right.",
            "3. Tap 'Excluded Applications'.",
            "4. Find and check 'PESU WiFi', then tap Save."
        ),
        lanNotice = "Local campus authentication will bypass the tunnel directly via physical Wi-Fi."
    ),
    VpnGuide(
        id = "warp",
        name = "Cloudflare 1.1.1.1 (WARP)",
        shortName = "WARP",
        packageNames = listOf("com.cloudflare.onedotonedotonedotone"),
        description = "Allow PESU WiFi to bypass Cloudflare WARP.",
        steps = listOf(
            "1. Open 1.1.1.1 and tap the menu (hamburger icon) in the top right.",
            "2. Go to 'Advanced' → 'Connection options'.",
            "3. Tap 'Exclude apps' and check 'PESU WiFi'."
        ),
        lanNotice = "WARP allows local networks by default, but app exclusion ensures direct gateway reachability."
    ),
    VpnGuide(
        id = "mullvad",
        name = "Mullvad VPN",
        shortName = "Mullvad",
        packageNames = listOf("net.mullvad.mullvadvpn"),
        description = "Bypass Mullvad VPN tunnel for PESU WiFi.",
        steps = listOf(
            "1. Open Mullvad and tap the Settings gear icon.",
            "2. Tap 'Split tunneling'.",
            "3. Find 'PESU WiFi' and tap the plus (+) icon to exclude it."
        ),
        lanNotice = "Under Settings → VPN settings, enable 'Local network sharing' if using killswitch."
    ),
    VpnGuide(
        id = "nordvpn",
        name = "NordVPN",
        shortName = "NordVPN",
        packageNames = listOf("com.nordvpn.android"),
        description = "Configure NordVPN Split Tunneling for PESU WiFi.",
        steps = listOf(
            "1. Open NordVPN and tap Profile → Settings gear.",
            "2. Tap 'Split tunneling' and turn it ON.",
            "3. Choose 'Disallow VPN for selected apps' and select 'PESU WiFi'."
        ),
        lanNotice = "Enable 'Local network discovery' in NordVPN Settings."
    ),
    VpnGuide(
        id = "surfshark",
        name = "Surfshark",
        shortName = "Surfshark",
        packageNames = listOf("com.surfshark.vpnclient.android"),
        description = "Use Surfshark Bypasser to exclude PESU WiFi.",
        steps = listOf(
            "1. Open Surfshark and go to Settings → Features.",
            "2. Tap 'Bypasser' and select 'Bypass VPN'.",
            "3. Add 'PESU WiFi' to the bypass list."
        ),
        lanNotice = "Enable 'Bypass local network' under VPN connection settings."
    ),
    VpnGuide(
        id = "adguard",
        name = "AdGuard / AdGuard VPN",
        shortName = "AdGuard",
        packageNames = listOf("com.adguard.android", "com.adguard.vpn"),
        description = "Exclude PESU WiFi from AdGuard filtering or VPN tunnel.",
        steps = listOf(
            "1. Open AdGuard and navigate to 'App Management'.",
            "2. Search for 'PESU WiFi'.",
            "3. Toggle OFF 'Route traffic through AdGuard' (or in AdGuard VPN → 'Exclusions' → add 'PESU WiFi')."
        ),
        lanNotice = "Allows raw direct socket binding without intermediate local proxy delays."
    ),
    VpnGuide(
        id = "openvpn",
        name = "OpenVPN (Connect / for Android)",
        shortName = "OpenVPN",
        packageNames = listOf("net.openvpn.openvpn", "de.blinkt.openvpn"),
        description = "Exclude PESU WiFi from OpenVPN tunnels.",
        steps = listOf(
            "1. Open OpenVPN and edit your profile / tunnel settings.",
            "2. Go to 'Allowed Apps' (or Settings → 'Split Tunneling').",
            "3. Exclude 'PESU WiFi' from the VPN tunnel."
        ),
        lanNotice = "Ensure 'Bypass VPN for local networks' is enabled in settings."
    )
)

@Composable
fun VpnCompatibilityDialog(
    onDismiss: () -> Unit
) {
    val context = LocalContext.current

    // Detect which known VPN apps are installed on the device
    val detectedGuides = remember(context) {
        VPN_GUIDES.filter { guide ->
            guide.packageNames.any { isPackageInstalled(context, it) }
        }
    }

    // Default to the first detected app if available, otherwise Universal Guide
    var selectedGuideId by remember {
        mutableStateOf(detectedGuides.firstOrNull()?.id ?: "universal")
    }

    val selectedGuide = remember(selectedGuideId) {
        VPN_GUIDES.firstOrNull { it.id == selectedGuideId } ?: VPN_GUIDES.first()
    }

    // Check if the selected guide has an installed package on this device
    val installedPackageForSelected = remember(selectedGuide, context) {
        selectedGuide.packageNames.firstOrNull { isPackageInstalled(context, it) }
    }

    AlertDialog(
        onDismissRequest = onDismiss,
        icon = {
            Icon(
                imageVector = Icons.Default.Security,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(32.dp)
            )
        },
        title = {
            Text(
                text = "VPN & Split Tunneling Guide",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
        },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
            ) {
                Text(
                    text = "A VPN is active on your device. Android full-tunnel VPNs route all traffic—including campus captive portal requests—into the tunnel by default, preventing connection to 192.168.254.1.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )

                Spacer(modifier = Modifier.height(10.dp))

                // Detected Apps highlight badge
                if (detectedGuides.isNotEmpty()) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(8.dp))
                            .background(StatusGreen.copy(alpha = 0.12f))
                            .padding(horizontal = 10.dp, vertical = 6.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            imageVector = Icons.Default.CheckCircle,
                            contentDescription = null,
                            tint = StatusGreen,
                            modifier = Modifier.size(16.dp)
                        )
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(
                            text = "Detected on your device: ${detectedGuides.joinToString(", ") { it.shortName }}",
                            style = MaterialTheme.typography.labelSmall,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.onSurface
                        )
                    }
                    Spacer(modifier = Modifier.height(10.dp))
                }

                // Filter Chips for App Selection
                Text(
                    text = "Select VPN App:",
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onSurface
                )
                Spacer(modifier = Modifier.height(6.dp))

                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .horizontalScroll(rememberScrollState()),
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    VPN_GUIDES.forEach { guide ->
                        val isDetected = detectedGuides.any { it.id == guide.id }
                        val isSelected = guide.id == selectedGuideId

                        FilterChip(
                            selected = isSelected,
                            onClick = { selectedGuideId = guide.id },
                            label = {
                                Text(
                                    text = if (isDetected) "${guide.shortName} ✓" else guide.shortName,
                                    fontSize = 12.sp,
                                    fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Normal
                                )
                            },
                            colors = FilterChipDefaults.filterChipColors(
                                selectedContainerColor = MaterialTheme.colorScheme.primaryContainer,
                                selectedLabelColor = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        )
                    }
                }

                Spacer(modifier = Modifier.height(12.dp))

                // Step-by-step instructions card for selected guide
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .background(MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f))
                        .padding(12.dp)
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = selectedGuide.name,
                            style = MaterialTheme.typography.labelLarge,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.primary
                        )
                        if (installedPackageForSelected != null) {
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(4.dp))
                                    .background(StatusGreen.copy(alpha = 0.2f))
                                    .padding(horizontal = 6.dp, vertical = 2.dp)
                            ) {
                                Text(
                                    text = "Installed",
                                    style = MaterialTheme.typography.labelSmall,
                                    fontSize = 10.sp,
                                    fontWeight = FontWeight.Bold,
                                    color = StatusGreen
                                )
                            }
                        }
                    }

                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = selectedGuide.description,
                        style = MaterialTheme.typography.bodySmall,
                        fontSize = 12.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )

                    Spacer(modifier = Modifier.height(8.dp))

                    selectedGuide.steps.forEach { step ->
                        Text(
                            text = step,
                            style = MaterialTheme.typography.bodySmall,
                            fontSize = 12.sp,
                            lineHeight = 17.sp,
                            color = MaterialTheme.colorScheme.onSurface
                        )
                        Spacer(modifier = Modifier.height(3.dp))
                    }

                    if (selectedGuide.lanNotice != null) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(6.dp))
                                .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.7f))
                                .padding(8.dp),
                            verticalAlignment = Alignment.Top
                        ) {
                            Icon(
                                imageVector = Icons.Default.Info,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary,
                                modifier = Modifier
                                    .size(16.dp)
                                    .padding(top = 1.dp)
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = selectedGuide.lanNotice,
                                style = MaterialTheme.typography.bodySmall,
                                fontSize = 11.sp,
                                lineHeight = 15.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }
                }

                Spacer(modifier = Modifier.height(10.dp))

                Text(
                    text = "Once PESU WiFi is added to your VPN's Split Tunnel, authentication succeeds directly over Wi-Fi with 0ms drops while all other app traffic remains encrypted inside your VPN tunnel.",
                    style = MaterialTheme.typography.bodySmall,
                    fontSize = 11.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        },
        confirmButton = {
            if (installedPackageForSelected != null) {
                Button(
                    onClick = {
                        val launchIntent = context.packageManager.getLaunchIntentForPackage(installedPackageForSelected)
                        if (launchIntent != null) {
                            context.startActivity(launchIntent)
                        }
                        onDismiss()
                    }
                ) {
                    Icon(
                        imageVector = Icons.Default.OpenInNew,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text("Open ${selectedGuide.shortName}")
                }
            } else {
                Button(
                    onClick = {
                        try {
                            context.startActivity(Intent(Settings.ACTION_VPN_SETTINGS))
                        } catch (_: Exception) {}
                        onDismiss()
                    }
                ) {
                    Icon(
                        imageVector = Icons.Default.Settings,
                        contentDescription = null,
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                    Text("VPN Settings")
                }
            }
        },
        dismissButton = {
            Row {
                if (installedPackageForSelected != null) {
                    OutlinedButton(
                        onClick = {
                            try {
                                context.startActivity(Intent(Settings.ACTION_VPN_SETTINGS))
                            } catch (_: Exception) {}
                            onDismiss()
                        }
                    ) {
                        Text("System VPN", fontSize = 12.sp)
                    }
                    Spacer(modifier = Modifier.width(6.dp))
                }
                TextButton(onClick = onDismiss) {
                    Text("Got It")
                }
            }
        }
    )
}

private fun isPackageInstalled(context: Context, packageName: String): Boolean {
    return try {
        context.packageManager.getPackageInfo(packageName, 0)
        true
    } catch (_: Exception) {
        false
    }
}
