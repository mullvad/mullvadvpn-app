package net.mullvad.mullvadvpn.service.notifications.old

// val tunnelStateChannel = NotificationChannel(
//    "vpn_tunnel_status",
//    NotificationCompat.VISIBILITY_SECRET,
//    R.string.foreground_notification_channel_name,
//    R.string.foreground_notification_channel_description,
//    false,
//    false
// )

// class NotificationChannelOld(
//    val context: Context,
//    val id: String,
//    val visibility: Int,
//    name: Int,
//    description: Int,
//    importance: Int,
//    isVibrationEnabled: Boolean,
//    isBadgeEnabled: Boolean
// ) {
//    private val badgeColor by lazy { context.getColor(R.color.colorPrimary) }
//
//    val notificationManager = NotificationManagerCompat.from(context)
//
//    init {
//        val channelName = context.getString(name)
//        val channelDescription = context.getString(description)
//
//        val channel =
//            NotificationChannelCompat.Builder(id, importance)
//                .setName(channelName)
//                .setDescription(channelDescription)
//                .setShowBadge(isBadgeEnabled)
//                .setVibrationEnabled(isVibrationEnabled)
//                .build()
//
//        notificationManager.createNotificationChannel(channel)
//    }
//
//    fun buildNotification(
//        intent: PendingIntent,
//        title: String,
//        deleteIntent: PendingIntent? = null,
//        isOngoing: Boolean = false
//    ): Notification {
//        return buildNotification(intent, title, emptyList(), deleteIntent, isOngoing)
//    }
//
//    fun buildNotification(
//        pendingIntent: PendingIntent,
//        title: Int,
//        actions: List<NotificationCompat.Action>,
//        deleteIntent: PendingIntent? = null,
//        isOngoing: Boolean = false
//    ): Notification {
//        return buildNotification(
//            pendingIntent,
//            context.getString(title),
//            actions,
//            deleteIntent,
//            isOngoing
//        )
//    }
//
//    private fun buildNotification(
//        pendingIntent: PendingIntent,
//        title: String,
//        actions: List<NotificationCompat.Action>,
//        deleteIntent: PendingIntent? = null,
//        isOngoing: Boolean = false
//    ): Notification {
//        val builder =
//            NotificationCompat.Builder(context, id)
//                .setSmallIcon(R.drawable.small_logo_black)
//                .setColor(badgeColor)
//                .setContentTitle(title)
//                .setContentIntent(pendingIntent)
//                .setVisibility(visibility)
//                .setOngoing(isOngoing)
//        for (action in actions) {
//            builder.addAction(action)
//        }
//
//        deleteIntent?.let { intent -> builder.setDeleteIntent(intent) }
//
//        return builder.build()
//    }
// }
