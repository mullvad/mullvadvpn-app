package net.mullvad.mullvadvpn.test.benchmark.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class IperfResult(
    val start: Start,
    val intervals: List<Interval>,
    val end: End,
)

@Serializable
data class Start(
    val connected: List<Connected>,
    val version: String,

    @SerialName("system_info")
    val systemInfo: String,
    val timestamp: Timestamp,
    @SerialName("connecting_to")
    val connectingTo: ConnectingTo,
    val cookie: String,
    @SerialName("tcp_mss_default")
    val tcpMssDefault: Long,
    @SerialName("target_bitrate")
    val targetBitrate: Long,
    @SerialName("fq_rate")
    val fqRate: Long,
    @SerialName("sock_bufsize")
    val sockBufsize: Long,
    @SerialName("sndbuf_actual")
    val sndbufActual: Long,
    @SerialName("rcvbuf_actual")
    val rcvbufActual: Long,
    @SerialName("test_start")
    val testStart: TestStart,
)

@Serializable
data class Connected(
    val socket: Long,
    @SerialName("local_host")
    val localHost: String,
    @SerialName("local_port")
    val localPort: Long,
    @SerialName("remote_host")
    val remoteHost: String,
    @SerialName("remote_port")
    val remotePort: Long,
)

@Serializable
data class Timestamp(
    val time: String,
    val timesecs: Long,
)

@Serializable
data class ConnectingTo(
    val host: String,
    val port: Long,
)

@Serializable
data class TestStart(
    val protocol: String,
    @SerialName("num_streams")
    val numStreams: Long,
    val blksize: Long,
    val omit: Long,
    val duration: Long,
    val bytes: Long,
    val blocks: Long,
    val reverse: Long,
    val tos: Long,
    @SerialName("target_bitrate")
    val targetBitrate: Long,
    val bidir: Long,
    val fqrate: Long,
    val interval: Long,
)

@Serializable
data class Interval(
    val streams: List<Stream>,
    val sum: Sum,
)

@Serializable
data class Stream(
    val socket: Long,
    val start: Double,
    val end: Double,
    val seconds: Double,
    val bytes: Long,
    @SerialName("bits_per_second")
    val bitsPerSecond: Double,
    val retransmits: Long,
    @SerialName("snd_cwnd")
    val sndCwnd: Long,
    @SerialName("snd_wnd")
    val sndWnd: Long,
    val rtt: Long,
    val rttvar: Long,
    val pmtu: Long,
    val omitted: Boolean,
    val sender: Boolean,
)

@Serializable
data class Sum(
    val start: Double,
    val end: Double,
    val seconds: Double,
    val bytes: Long,
    @SerialName("bits_per_second")
    val bitsPerSecond: Double,
    val retransmits: Long,
    val omitted: Boolean,
    val sender: Boolean,
)

@Serializable
data class End(
    val streams: List<Stream2>,
    @SerialName("sum_sent")
    val sumSent: SumSent,
    @SerialName("sum_received")
    val sumReceived: SumReceived,
    @SerialName("cpu_utilization_percent")
    val cpuUtilizationPercent: CpuUtilizationPercent,
    @SerialName("sender_tcp_congestion")
    val senderTcpCongestion: String,
    @SerialName("receiver_tcp_congestion")
    val receiverTcpCongestion: String,
)

@Serializable
data class Stream2(
    val sender: Sender,
    val receiver: Receiver,
)

@Serializable
data class Sender(
    val socket: Long,
    val start: Long,
    val end: Double,
    val seconds: Double,
    val bytes: Long,
    @SerialName("bits_per_second")
    val bitsPerSecond: Double,
    val retransmits: Long,
    @SerialName("max_snd_cwnd")
    val maxSndCwnd: Long,
    @SerialName("max_snd_wnd")
    val maxSndWnd: Long,
    @SerialName("max_rtt")
    val maxRtt: Long,
    @SerialName("min_rtt")
    val minRtt: Long,
    @SerialName("mean_rtt")
    val meanRtt: Long,
    val sender: Boolean,
)

@Serializable
data class Receiver(
    val socket: Long,
    val start: Long,
    val end: Double,
    val seconds: Double,
    val bytes: Long,
    @SerialName("bits_per_second")
    val bitsPerSecond: Double,
    val sender: Boolean,
)

@Serializable
data class SumSent(
    val start: Long,
    val end: Double,
    val seconds: Double,
    val bytes: Long,
    @SerialName("bits_per_second")
    val bitsPerSecond: Double,
    val retransmits: Long,
    val sender: Boolean,
)

@Serializable
data class SumReceived(
    val start: Long,
    val end: Double,
    val seconds: Double,
    val bytes: Long,
    @SerialName("bits_per_second")
    val bitsPerSecond: Double,
    val sender: Boolean,
)

@Serializable
data class CpuUtilizationPercent(
    @SerialName("host_total")
    val hostTotal: Double,
    @SerialName("host_user")
    val hostUser: Double,
    @SerialName("host_system")
    val hostSystem: Double,
    @SerialName("remote_total")
    val remoteTotal: Double,
    @SerialName("remote_user")
    val remoteUser: Double,
    @SerialName("remote_system")
    val remoteSystem: Double,
)
