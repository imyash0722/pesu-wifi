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

        @Volatile
        var isVpnPermFailureDetected: Boolean = false

        fun reset() {
            isVpnPermFailureDetected = false
        }

        fun isPermOrBindingFailure(t: Throwable): Boolean {
            if (t is SecurityException) return true
            val msg = t.message?.lowercase() ?: ""
            return msg.contains("eperm") ||
                msg.contains("operation not permitted") ||
                msg.contains("permission denied") ||
                msg.contains("binding socket") ||
                msg.contains("failed to bind")
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
     * thrown when an active VPN disallows direct physical interface binding) and dynamically falls back
     * to a standard system socket to reach local RFC-1918 gateway IPs.
     */
    class ResilientSocket(
        private val primaryFactory: SocketFactory?,
        private val fallbackFactory: SocketFactory,
        private val localBindAddress: InetAddress?
    ) : Socket() {

        private var activeSocket: Socket = initializeSocket()

        private fun initializeSocket(): Socket {
            return if (!isVpnPermFailureDetected && primaryFactory != null) {
                try {
                    primaryFactory.createSocket()
                } catch (t: Throwable) {
                    if (isPermOrBindingFailure(t)) {
                        isVpnPermFailureDetected = true
                        AppLogger.w(TAG, "Wi-Fi socket creation threw ${t.message} (VPN active); using standard socket")
                        createFallbackSocket()
                    } else if (t is IOException) {
                        throw t
                    } else {
                        throw IOException("Socket creation failed", t)
                    }
                }
            } else {
                createFallbackSocket()
            }
        }

        private fun createFallbackSocket(): Socket {
            val s = fallbackFactory.createSocket()
            if (localBindAddress != null) {
                try {
                    s.bind(InetSocketAddress(localBindAddress, 0))
                } catch (e: Exception) {
                    AppLogger.d(TAG, "Fallback socket bind to $localBindAddress skipped: ${e.message}")
                }
            }
            return s
        }

        override fun connect(endpoint: SocketAddress?) {
            connect(endpoint, 0)
        }

        override fun connect(endpoint: SocketAddress?, timeout: Int) {
            try {
                activeSocket.connect(endpoint, timeout)
            } catch (t: Throwable) {
                if (isPermOrBindingFailure(t)) {
                    isVpnPermFailureDetected = true
                    AppLogger.w(TAG, "Wi-Fi bound socket failed connect with ${t.message} (VPN active). Transparently falling back to standard system socket...")
                    try {
                        activeSocket.close()
                    } catch (_: Exception) {}

                    val fallback = fallbackFactory.createSocket()
                    fallback.connect(endpoint, timeout)
                    activeSocket = fallback
                    AppLogger.i(TAG, "VPN fallback socket connected successfully to $endpoint")
                } else if (t is IOException) {
                    throw t
                } else {
                    throw IOException("Connection failed to $endpoint", t)
                }
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
