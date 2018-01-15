package com.mullvad;

import android.app.Notification;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.net.VpnService;
import android.os.AsyncTask;
import android.os.Binder;
import android.os.Bundle;
import android.os.IBinder;
import android.os.ParcelFileDescriptor;
import android.util.Log;

import wireguardbinding.Wireguardbinding;

import com.mullvad.ServicePackage.MobileAppBridge;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;

public class WireGuardVpnService extends VpnService {

    private static final String TAG = "WireGuardVpnService";
    private static final int NOTIFICATION_ID = 1;
    public enum State {SECURE, INSECURE};
    public static final String ACTION = "ACTION";
    public enum Action {STOP, START, EXIT};
    private static final int ACTIVITY_REQUEST_CODE = 1;
    private static final int ACTION_REQUEST_CODE = 2;
    private static final int EXIT_REQUEST_CODE = 3;

    private final IBinder binder = new Binder();
    private boolean started = false;

    private Process logProcess;
    public static final String LOG_FILE_PATH = "/logs";
    public static final String LOG_FILE_NAME = "log.log";

    static {
        System.loadLibrary("mullvad");
    }

    private static native String toBackend(String json);

    protected void toFrontend(String json){
        sendMessageToFrontend(MobileAppBridge.ACTION_MESSAGE,json);
    }

    private void sendMessageToFrontend(String action,String message){
        sendBroadcast(new Intent(MobileAppBridge.BRIDGE_FILTER).setAction(action).putExtra(MobileAppBridge.MESSAGE,message));
    }

    @Override
    public void onCreate() {
        super.onCreate();
        startLogging();
        Log.d(TAG,"onCreate");
        setNotification(State.INSECURE);
        sendMessageToFrontend(MobileAppBridge.ACTION_BACKEND_INFO,MobileAppBridge.BACKEND_INFO);
    }

    @Override
    public void onDestroy(){
        super.onDestroy();
        stopLogging();
    }

    private void stopLogging(){
        if (logProcess != null){
            logProcess.destroy();
        }
    }

    private void startLogging(){
        Log.d(TAG,getFilesDir().getAbsolutePath());
        File outputFile = new File(getFilesDir(),LOG_FILE_NAME);
        Log.d(TAG,outputFile.getAbsolutePath());
        try {
            logProcess = Runtime.getRuntime().exec("logcat -f "+outputFile.getAbsolutePath());
        } catch (IOException e) {
            e.printStackTrace();
        }
    }


    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        if (intent != null){
            Bundle bundle = intent.getExtras();
            if (bundle != null) {
                for (String key : bundle.keySet()) {
                    Object value = bundle.get(key);
                    Log.d(TAG, String.format("%s %s (%s)", key,
                            value.toString(), value.getClass().getName()));
                }
            }

            if (intent.hasExtra(ACTION)){
                switch ((Action)intent.getSerializableExtra(ACTION)){
                    case EXIT:
                        Log.d(TAG,Action.EXIT.name());
                        Wireguardbinding.stop();
                        stopSelf();
                        break;
                    case STOP:
                        Log.d(TAG,Action.STOP.name());
                        disable();
                        break;
                    case START:
                        Log.d(TAG,Action.START.name());
                        enable();
                        break;
                }
            } else if (intent.hasExtra(MobileAppBridge.MESSAGE)){
                toBackend(intent.getStringExtra(MobileAppBridge.MESSAGE));
            }

            return START_STICKY;
        }
        return START_NOT_STICKY;
    }

    private void setNotification(State state){
        Intent notificationIntent = new Intent(this, MainActivity.class);
        PendingIntent pendingIntent =
                PendingIntent.getActivity(this, ACTIVITY_REQUEST_CODE, notificationIntent, PendingIntent.FLAG_UPDATE_CURRENT);

        Intent actionIntent = new Intent(this, WireGuardVpnService.class).putExtra(ACTION,state == State.SECURE ? Action.STOP : Action.START);
        PendingIntent actionPendingIntent =
                PendingIntent.getService(this, ACTION_REQUEST_CODE, actionIntent, PendingIntent.FLAG_UPDATE_CURRENT);
        Notification.Action action = new Notification.Action.Builder(state == State.SECURE ? R.drawable.ic_unlocked : R.drawable.ic_locked,state == State.SECURE ? "Stop" : "Start",actionPendingIntent).build();

        Intent exitIntent = new Intent(this, WireGuardVpnService.class).putExtra(ACTION,Action.EXIT);
        PendingIntent exitPendingIntent =
                PendingIntent.getService(this, EXIT_REQUEST_CODE, exitIntent, PendingIntent.FLAG_UPDATE_CURRENT);
        Notification.Action exitAction = new Notification.Action.Builder(R.drawable.icon_close,"Exit",exitPendingIntent).build();

        Notification notification =
                new Notification.Builder(this)
                        .setContentTitle(getText(R.string.vpn_notification_title))
                        .setContentText(state == State.SECURE ? getText(R.string.vpn_notification_message_secured) : getText(R.string.vpn_notification_message_insecure))
                        .setSmallIcon(state == State.SECURE ? R.drawable.ic_locked : R.drawable.ic_unlocked)
                        .setContentIntent(pendingIntent)
                        .setOngoing(true)
                        .setTicker(state == State.SECURE ? getText(R.string.vpn_ticker_text_secured) : getText(R.string.vpn_ticker_text_unsecured))
                        .addAction(action)
                        .addAction(exitAction)
                        .build();

        //Set service to foreground with persistent notification
        startForeground(NOTIFICATION_ID, notification);
    }

    public void disable() {
        new ConfigDisabler(null).execute();
    }

    public void enable() {
        new ConfigEnabler(null).execute();
    }


    @Override
    public IBinder onBind(Intent intent) {
        Log.d(TAG,"onBind");
        //Will probably not be used
        return binder;
    }

    private class ConfigDisabler extends AsyncTask<Void, Void, Boolean> {
        private final String config;

        private ConfigDisabler(final String config) {
            this.config = config;
        }

        @Override
        protected Boolean doInBackground(final Void... voids) {
            Wireguardbinding.stop();
            return true;
        }

        @Override
        protected void onPostExecute(final Boolean result) {
            if (!result)
                return;
            sendMessageToFrontend(MobileAppBridge.ACTION_MESSAGE,"Disabled");
            setNotification(State.INSECURE);
        }
    }

    private class ConfigEnabler extends AsyncTask<Void, Void, Boolean> {
        private final String config;

        private ConfigEnabler(final String config) {
            this.config = config;
        }

        @Override
        protected Boolean doInBackground(final Void... voids) {
            // Vpn service need to be already ready
            if(prepare(getBaseContext()) != null)
                return false;

            Builder builder = new Builder();

            builder.setSession("Mullvad VPN");

            builder.addRoute("0.0.0.0", 0);
            builder.setBlocking(true);
            ParcelFileDescriptor tun = builder.establish();
            if (tun == null) {
                Log.d(TAG, "Unable to create tun device");
                return false;
            }

            Wireguardbinding.start(tun.detachFd(), "Mullvad VPN");
            long socket = 0;
            while((socket = Wireguardbinding.socket()) == 0) {
                Log.d(TAG, "Wait for socket");
                try {
                    Thread.sleep(1000);
                } catch (InterruptedException e) {
                    e.printStackTrace();
                }
            }

            protect((int) socket);

            return true;
        }

        @Override
        protected void onPostExecute(final Boolean result) {
            if (!result)
                return;
            sendMessageToFrontend(MobileAppBridge.ACTION_MESSAGE,"Enabled");
            setNotification(State.SECURE);
        }
    }

}
