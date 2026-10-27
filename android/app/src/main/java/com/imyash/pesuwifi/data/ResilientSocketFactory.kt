package com.imyash.pesuwifi.data

import com.imyash.pesuwifi.util.AppLogger
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket
import java.net.SocketAddress
import java.nio.channels.SocketChannel
import javax.net.SocketFactory

/**
 * Resilient SocketFactory wrapper that attempts to route through a target network's SocketFactory
 * (such as Wi-Fi Network.socketFactory) and gracefully falls back to the system default SocketFactory
 * if the kernel blocks socket binding (e.g., EPERM / Operation not permitted when a VPN is active).
 *
 * Catches EPERM both at socket creation time and during connect(), enabling seamless operation
 * alongside active VPN tunnels (Tailscale, WireGuard, Cloudflare WARP, ProtonVPN, OpenVPN).
 */
class ResilientSocketFactory(
    private val delegate: SocketFactory?,
    private val localBindAddress: InetAddress? = null
) : SocketFactory() {

    private val defaultFactory: SocketFactory = getDefault()

    companion object {
        private const val TAG = "ResilientSocket"

        fun isPermOrBindingFailure(t: Throwable): Boolean {
            if (t is SecurityException) return true
            val msg = t.message?.lowercase() ?: ""
            return msg.contains("eperm") ||
                msg.contains("operation not permitted") ||
                msg.contains("permission denied") ||
                msg.contains("binding socket") ||
                msg.contains("failed to bind") ||
                msg.contains("eacces") ||
                msg.contains("enonet") ||
                msg.contains("machine is not on the network") ||
                msg.contains("network is down") ||
                msg.contains("network is unreachable") ||
                msg.contains("no route to host") ||
                msg.contains("socket failed")
        }
    }

    override fun createSocket(): Socket {
        return ResilientSocket(delegate, defaultFactory, localBindAddress)
    }

    override fun createSocket(host: String, port: Int): Socket {
        val s = createSocket()
        s.connect(InetSocketAddress(host, port), 5000)
        return s
    }

    override fun createSocket(host: String, port: Int, localHost: InetAddress, localPort: Int): Socket {
        val s = createSocket()
        try {
            s.bind(InetSocketAddress(localHost, localPort))
        } catch (e: Exception) {
            AppLogger.w(TAG, "Local bind failed in createSocket: ${e.message}")
        }
        s.connect(InetSocketAddress(host, port), 5000)
        return s
    }

    override fun createSocket(host: InetAddress, port: Int): Socket {
        val s = createSocket()
        s.connect(InetSocketAddress(host, port), 5000)
        return s
    }

    override fun createSocket(address: InetAddress, port: Int, localAddress: InetAddress, localPort: Int): Socket {
        val s = createSocket()
        try {
            s.bind(InetSocketAddress(localAddress, localPort))
        } catch (e: Exception) {
            AppLogger.w(TAG, "Local bind failed in createSocket: ${e.message}")
        }
        s.connect(InetSocketAddress(address, port), 5000)
        return s
    }

    /**
     * Delegating Socket wrapper that intercepts connect() errors (such as EPERM / Operation not permitted
     * or ENONET thrown when an active VPN disallows direct physical interface binding) and dynamically falls back
     * across a 3-tier escalation strategy:
     *   - Tier 1: Primary network-bound socket (via Network.socketFactory)
     *   - Tier 2: Local IP-bound system socket (via localBindAddress)
     *   - Tier 3: Completely unbound standard system socket (bypasses kernel routing/bind lockouts)
     */
    class ResilientSocket(
        private val primaryFactory: SocketFactory?,
        private val fallbackFactory: SocketFactory,
        private val localBindAddress: InetAddress?
    ) : Socket() {

        private var currentTier: Int = 1
        private var activeSocket: Socket = initializeSocket()

        private fun initializeSocket(): Socket {
            if (primaryFactory != null) {
                try {
                    currentTier = 1
                    return primaryFactory.createSocket()
                } catch (t: Throwable) {
                    AppLogger.w(TAG, "Primary Wi-Fi socket creation threw ${t.message}; falling back to local-bound/unbound socket")
                }
            }
            return createTier2Or3Socket()
        }

        private fun createTier2Or3Socket(): Socket {
            if (localBindAddress != null) {
                try {
                    val s = fallbackFactory.createSocket()
                    s.bind(InetSocketAddress(localBindAddress, 0))
                    currentTier = 2
                    return s
                } catch (e: Exception) {
                    AppLogger.d(TAG, "Fallback socket bind to $localBindAddress skipped: ${e.message}")
                }
            }
            currentTier = 3
            return fallbackFactory.createSocket()
        }

        private fun transferOptions(source: Socket, target: Socket) {
            try { target.tcpNoDelay = source.tcpNoDelay } catch (_: Exception) {}
            try { target.soTimeout = source.soTimeout } catch (_: Exception) {}
            try { target.keepAlive = source.keepAlive } catch (_: Exception) {}
            try { target.reuseAddress = source.reuseAddress } catch (_: Exception) {}
        }

        override fun connect(endpoint: SocketAddress?) {
            connect(endpoint, 0)
        }

        override fun connect(endpoint: SocketAddress?, timeout: Int) {
            // Tier 1 (or initialized tier): Try activeSocket
            try {
                activeSocket.connect(endpoint, timeout)
                return
            } catch (t: Throwable) {
                if (!isPermOrBindingFailure(t)) {
                    if (t is IOException) throw t else throw IOException("Connection failed to $endpoint", t)
                }
                AppLogger.w(TAG, "Socket tier $currentTier failed connect with ${t.message} (VPN active). Falling back...")
                try { activeSocket.close() } catch (_: Exception) {}
            }

            // Tier 2: Try local-bound fallback socket if we were on Tier 1
            if (currentTier == 1 && localBindAddress != null) {
                try {
                    val s = fallbackFactory.createSocket()
                    transferOptions(activeSocket, s)
                    try {
                        s.bind(InetSocketAddress(localBindAddress, 0))
                        currentTier = 2
                        s.connect(endpoint, timeout)
                        activeSocket = s
                        AppLogger.i(TAG, "Tier 2 local-bound fallback socket connected successfully to $endpoint")
                        return
                    } catch (t2: Throwable) {
                        try { s.close() } catch (_: Exception) {}
                        if (!isPermOrBindingFailure(t2)) {
                            if (t2 is IOException) throw t2 else throw IOException("Connection failed to $endpoint", t2)
                        }
                        AppLogger.w(TAG, "Tier 2 local-bound socket failed connect with ${t2.message}. Falling back to Tier 3 unbound socket...")
                    }
                } catch (e: Exception) {
                    AppLogger.d(TAG, "Tier 2 fallback socket setup skipped: ${e.message}")
                }
            }

            // Tier 3: Completely unbound standard system socket (bypasses kernel bind/protect restrictions)
            try {
                val unbound = fallbackFactory.createSocket()
                transferOptions(activeSocket, unbound)
                currentTier = 3
                unbound.connect(endpoint, timeout)
                activeSocket = unbound
                AppLogger.i(TAG, "Tier 3 unbound system socket connected successfully to $endpoint")
            } catch (t3: Throwable) {
                try { activeSocket.close() } catch (_: Exception) {}
                if (t3 is IOException) throw t3 else throw IOException("All socket tiers failed to connect to $endpoint", t3)
            }
        }

        override fun getInputStream(): InputStream = activeSocket.inputStream
        override fun getOutputStream(): OutputStream = activeSocket.outputStream
        override fun close() = activeSocket.close()
        override fun isConnected(): Boolean = activeSocket.isConnected
        override fun isBound(): Boolean = activeSocket.isBound
        override fun isClosed(): Boolean = activeSocket.isClosed

        override fun setSoTimeout(timeout: Int) { activeSocket.soTimeout = timeout }
        override fun getSoTimeout(): Int = activeSocket.soTimeout

        override fun setTcpNoDelay(on: Boolean) { activeSocket.tcpNoDelay = on }
        override fun getTcpNoDelay(): Boolean = activeSocket.tcpNoDelay

        override fun setReuseAddress(on: Boolean) { activeSocket.reuseAddress = on }
        override fun getReuseAddress(): Boolean = activeSocket.reuseAddress

        override fun getChannel(): SocketChannel? = activeSocket.channel
        override fun getLocalSocketAddress(): SocketAddress? = activeSocket.localSocketAddress
        override fun getRemoteSocketAddress(): SocketAddress? = activeSocket.remoteSocketAddress
        override fun getInetAddress(): InetAddress? = activeSocket.inetAddress
        override fun getPort(): Int = activeSocket.port
        override fun getLocalPort(): Int = activeSocket.localPort

        override fun setSendBufferSize(size: Int) { activeSocket.sendBufferSize = size }
        override fun getSendBufferSize(): Int = activeSocket.sendBufferSize

        override fun setReceiveBufferSize(size: Int) { activeSocket.receiveBufferSize = size }
        override fun getReceiveBufferSize(): Int = activeSocket.receiveBufferSize

        override fun setKeepAlive(on: Boolean) { activeSocket.keepAlive = on }
        override fun getKeepAlive(): Boolean = activeSocket.keepAlive

        override fun shutdownInput() = activeSocket.shutdownInput()
        override fun shutdownOutput() = activeSocket.shutdownOutput()
        override fun isInputShutdown(): Boolean = activeSocket.isInputShutdown
        override fun isOutputShutdown(): Boolean = activeSocket.isOutputShutdown
    }
}
