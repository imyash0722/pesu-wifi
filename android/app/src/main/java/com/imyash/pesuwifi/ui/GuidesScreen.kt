package com.imyash.pesuwifi.ui

import android.content.Intent
import android.net.Uri
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.OpenInBrowser
import androidx.compose.material.icons.filled.QuestionMark
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Security
import androidx.compose.material.icons.filled.Smartphone
import androidx.compose.material.icons.filled.Speed
import androidx.compose.material.icons.filled.SwitchAccount
import androidx.compose.material.icons.filled.SystemUpdate
import androidx.compose.material.icons.filled.VpnKey
import androidx.compose.material.icons.filled.Wifi
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.imyash.pesuwifi.BuildConfig
import com.imyash.pesuwifi.ui.components.VpnCompatibilityDialog
import com.imyash.pesuwifi.ui.theme.StatusGreen

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GuidesScreen(
    onNavigateBack: () -> Unit,
    onCheckForUpdates: () -> Unit = {}
) {
    val context = LocalContext.current
    val expandedStates = remember {
        mutableStateMapOf(
            "quickstart" to true,
            "gateways" to false,
            "vpn" to false,
            "accounts" to false,
            "ios" to false
        )
    }
    var showVpnDialog by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = "In-App Guides & Help",
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.SemiBold
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = "Back"
                        )
                    }
                },
                actions = {
                    IconButton(onClick = onCheckForUpdates) {
                        Icon(
                            imageVector = Icons.Default.SystemUpdate,
                            contentDescription = "Check for Updates"
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background
                )
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(horizontal = 20.dp)
                .verticalScroll(rememberScrollState())
        ) {
            Spacer(modifier = Modifier.height(12.dp))

            // ── Hero Banner Card ──
            Card(
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(16.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5f)
                )
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Box(
                        modifier = Modifier
                            .size(44.dp)
                            .clip(CircleShape)
                            .background(StatusGreen.copy(alpha = 0.15f)),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            imageVector = Icons.Default.Wifi,
                            contentDescription = null,
                            tint = StatusGreen,
                            modifier = Modifier.size(24.dp)
                        )
                    }
                    Spacer(modifier = Modifier.width(14.dp))
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = "PESU WiFi Knowledge Base",
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold
                        )
                        Text(
                            text = "Zero-drop background architecture, VPN bypass, and campus AP roaming guide.",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(20.dp))

            // ── Guide Section 1: Quick Start & Permissions ──
            GuideCard(
                title = "1. Quick Start & Background Permissions",
                subtitle = "Keepalive, Doze sleep immunity, and Quick Settings Tile",
                icon = Icons.Default.Speed,
                isExpanded = expandedStates["quickstart"] ?: false,
                onToggle = { expandedStates["quickstart"] = !(expandedStates["quickstart"] ?: false) }
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    GuideBullet(
                        title = "Zero-Fiddling Automation",
                        desc = "Once you enter your username & password and grant permissions, the app runs 24/7 in the background without needing to be opened."
                    )
                    GuideBullet(
                        title = "Foreground Service ('specialUse')",
                        desc = "Uses Android 14+ specialUse foreground service with continuous RTC hardware alarms, exempting the app from Android 15/16's 6-hour cumulative dataSync kill switch."
                    )
                    GuideBullet(
                        title = "Battery Optimization Whitelist",
                        desc = "Exempting the app from battery optimization allows the CPU to wake for 300ms every 120s to send keepalive pings without being killed in deep sleep."
                    )
                    GuideBullet(
                        title = "Quick Settings Pull-Down Tile",
                        desc = "Swipe down your Android notification shade, tap Edit Tiles, and drag 'PESU WiFi' to your active tiles for 1-tap connection toggle anytime."
                    )
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            // ── Guide Section 2: Gateways & Campus Roaming ──
            GuideCard(
                title = "2. Campus Gateways & Roaming",
                subtitle = "EC Campus, RR Campus & Amaatra Hostel",
                icon = Icons.Default.Wifi,
                isExpanded = expandedStates["gateways"] ?: false,
                onToggle = { expandedStates["gateways"] = !(expandedStates["gateways"] ?: false) }
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    GuideBullet(
                        title = "PESU EC & RR Campus",
                        desc = "Subnets: 10.x.x.x / 192.168.254.x. Central Cyberroam portal is located at http://192.168.254.1:8090."
                    )
                    GuideBullet(
                        title = "Amaatra Hostel",
                        desc = "Subnet: 172.16.x.x. Hostel gateway is located at http://172.16.1.1:8090. The app dynamically detects this gateway automatically."
                    )
                    GuideBullet(
                        title = "Strongest AP Roaming & Cisco Twins",
                        desc = "The app scans campus BSSIDs, prioritizes the highest RSSI signal, and automatically derives Cisco 5 GHz twin access points by flipping bit 6 of the 4th octet for seamless hallway roaming."
                    )
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            // ── Guide Section 3: Full VPN Compatibility & Split Tunneling ──
            GuideCard(
                title = "3. Full VPN Compatibility & Split Tunneling",
                subtitle = "Tailscale, Proton, WireGuard, WARP, Mullvad & More",
                icon = Icons.Default.VpnKey,
                isExpanded = expandedStates["vpn"] ?: false,
                onToggle = { expandedStates["vpn"] = !(expandedStates["vpn"] ?: false) }
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    GuideBullet(
                        title = "Universal 3-Step Rule (Any VPN App)",
                        desc = "1. Open your VPN app and go to Settings or Features.\n2. Look for 'Split Tunneling', 'Bypasser', or 'Excluded Apps'.\n3. Set mode to 'Exclude / Bypass' and check 'PESU WiFi'.\n• Strict Killswitch: Ensure 'Allow LAN / Local Network traffic' is toggled ON so campus portal probes reach 192.168.254.1."
                    )
                    GuideBullet(
                        title = "Tailscale (Self-Hosters & Exit Nodes)",
                        desc = "Tailscale captures private subnets by default. For seamless 24/7 self-hosting without ever turning Tailscale off:\n• In Tailscale → Settings → App Split Tunnel: check 'PESU WiFi'.\n• If using an Exit Node: enable 'Allow LAN access'.\nYour self-hosted traffic remains 100% inside Tailscale while portal authentication connects directly over Wi-Fi."
                    )
                    GuideBullet(
                        title = "Proton VPN, WireGuard & Cloudflare WARP",
                        desc = "• Proton VPN: Settings → Features → Split Tunneling (ON) → Exclude 'PESU WiFi'. Enable Settings → Advanced → 'Allow LAN connections'.\n• WireGuard: Edit Tunnel → Excluded Applications → Check 'PESU WiFi' → Save.\n• Cloudflare 1.1.1.1 (WARP): Menu → Advanced → Connection options → Exclude apps → Add 'PESU WiFi'."
                    )
                    GuideBullet(
                        title = "Mullvad, NordVPN, Surfshark & AdGuard",
                        desc = "• Mullvad: Settings → Split tunneling → Exclude 'PESU WiFi'. Enable 'Local network sharing'.\n• NordVPN: Profile → Settings → Split tunneling (ON) → Disallow for 'PESU WiFi'. Enable 'Local network discovery'.\n• Surfshark: Settings → Features → Bypasser → Bypass VPN → Add 'PESU WiFi'.\n• AdGuard: App Management → 'PESU WiFi' → Turn OFF 'Route traffic through AdGuard'."
                    )
                    GuideBullet(
                        title = "Desktop (Linux & Windows) - Zero Configuration",
                        desc = "On desktop, PESU WiFi automatically binds /32 host routes for campus gateways (192.168.254.1/32, 172.16.1.1/32) directly to your physical Wi-Fi interface. No manual split tunneling needed on PC."
                    )

                    Spacer(modifier = Modifier.height(10.dp))

                    Button(
                        onClick = { showVpnDialog = true },
                        modifier = Modifier.fillMaxWidth(),
                        colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primaryContainer)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Security,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onPrimaryContainer,
                            modifier = Modifier.size(16.dp)
                        )
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(
                            text = "Interactive VPN Guide & Detected Apps",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onPrimaryContainer,
                            fontWeight = FontWeight.Bold
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            // ── Guide Section 4: Multi-Account Auto-Failover ──
            GuideCard(
                title = "4. Smart Multi-Account Auto-Failover",
                subtitle = "Survive data limits & concurrent device blocks",
                icon = Icons.Default.SwitchAccount,
                isExpanded = expandedStates["accounts"] ?: false,
                onToggle = { expandedStates["accounts"] = !(expandedStates["accounts"] ?: false) }
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    GuideBullet(
                        title = "Quota & Limit Protection",
                        desc = "Campus Cyberroam limits concurrent active logins per account and imposes daily data caps. Store backup student or faculty accounts in the Accounts tab."
                    )
                    GuideBullet(
                        title = "Silent Auto-Switching",
                        desc = "If an active account hits 'Maximum Login Limit Reached' or 'Data limit exceeded', the background engine automatically switches to your next saved account without dropping your connection."
                    )
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            // ── Guide Section 5: iOS Companion Shortcuts ──
            GuideCard(
                title = "5. iOS Companion Guide (iPhone & iPad)",
                subtitle = "Zero-click Apple Shortcuts for your iOS devices",
                icon = Icons.Default.Smartphone,
                isExpanded = expandedStates["ios"] ?: false,
                onToggle = { expandedStates["ios"] = !(expandedStates["ios"] ?: false) }
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "Have an iPhone, iPad, or Mac? Apple Shortcuts provides 100% native background auto-login on iOS:",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(10.dp))
                    GuideBullet(
                        title = "1-Tap Shortcut Generator",
                        desc = "Visit the web guide to download a ready-made .shortcut file with your credentials pre-packaged."
                    )
                    GuideBullet(
                        title = "Personal Automation",
                        desc = "Set Shortcuts Automation to trigger on Wi-Fi connect with 'Run Immediately' enabled."
                    )
                    Spacer(modifier = Modifier.height(6.dp))
                    Button(
                        onClick = {
                            val intent = Intent(Intent.ACTION_VIEW, Uri.parse("https://imyash0722.github.io/pesu-wifi/ios.html"))
                            context.startActivity(intent)
                        },
                        colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.primary)
                    ) {
                        Icon(
                            imageVector = Icons.Default.OpenInBrowser,
                            contentDescription = null,
                            modifier = Modifier.size(16.dp)
                        )
                        Spacer(modifier = Modifier.width(6.dp))
                        Text(text = "Open iOS Shortcuts Web Guide", fontSize = 12.sp)
                    }
                }
            }

            Spacer(modifier = Modifier.height(24.dp))

            // ── In-App Updater & About Card ──
            Card(
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(16.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.35f)
                )
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(
                            modifier = Modifier
                                .weight(1f)
                                .padding(end = 12.dp)
                        ) {
                            Text(
                                text = "PESU WiFi for Android",
                                style = MaterialTheme.typography.titleMedium,
                                fontWeight = FontWeight.Bold
                            )
                            Text(
                                text = "Version ${BuildConfig.VERSION_NAME} (code ${BuildConfig.VERSION_CODE})",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.outline
                            )
                        }

                        Button(
                            onClick = onCheckForUpdates,
                            shape = RoundedCornerShape(50.dp),
                            contentPadding = PaddingValues(horizontal = 14.dp, vertical = 8.dp),
                            colors = ButtonDefaults.buttonColors(
                                containerColor = StatusGreen,
                                contentColor = Color.White
                            )
                        ) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Icon(
                                    imageVector = Icons.Default.SystemUpdate,
                                    contentDescription = null,
                                    modifier = Modifier.size(16.dp)
                                )
                                Spacer(modifier = Modifier.width(6.dp))
                                Text(
                                    text = "Check Updates",
                                    fontSize = 12.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    maxLines = 1,
                                    softWrap = false
                                )
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(32.dp))
        }

        if (showVpnDialog) {
            VpnCompatibilityDialog(
                onDismiss = { showVpnDialog = false }
            )
        }
    }
}

@Composable
private fun GuideCard(
    title: String,
    subtitle: String,
    icon: ImageVector,
    isExpanded: Boolean,
    onToggle: () -> Unit,
    content: @Composable () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .clickable { onToggle() },
        shape = RoundedCornerShape(16.dp),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
        )
    ) {
        Column {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Box(
                    modifier = Modifier
                        .size(38.dp)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.12f)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(20.dp)
                    )
                }
                Spacer(modifier = Modifier.width(12.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = title,
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = subtitle,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                Icon(
                    imageVector = if (isExpanded) Icons.Default.ExpandMore else Icons.Default.ChevronRight,
                    contentDescription = if (isExpanded) "Collapse" else "Expand",
                    tint = MaterialTheme.colorScheme.outline
                )
            }

            AnimatedVisibility(
                visible = isExpanded,
                enter = expandVertically() + fadeIn(),
                exit = shrinkVertically() + fadeOut()
            ) {
                Column {
                    HorizontalDivider(
                        color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3f)
                    )
                    content()
                }
            }
        }
    }
}

@Composable
private fun GuideBullet(
    title: String,
    desc: String
) {
    Column(modifier = Modifier.padding(bottom = 12.dp)) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.Bold,
            color = MaterialTheme.colorScheme.primary
        )
        Spacer(modifier = Modifier.height(2.dp))
        Text(
            text = desc,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            lineHeight = 18.sp
        )
    }
}
