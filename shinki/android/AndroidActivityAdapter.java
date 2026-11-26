package org.shinkirou.krkr;

import android.app.Activity;

/** Connects the core's Java services to the platform Activity lifecycle. */
public final class AndroidActivityAdapter implements org.shinkirou.shinkiemu.AndroidActivityAdapter {
    @Override
    public void attached(Activity activity) {
        KrkrHost.prepareActivity(activity);
    }

    @Override
    public void resumed() {
        KrkrHost.onResume();
    }

    @Override
    public void paused() {
        KrkrHost.onPause();
    }

    @Override
    public void detaching(Activity activity) {
        KrkrHost.cancelActivation(activity);
    }

    @Override
    public void detached(Activity activity) {
        KrkrHost.destroy(activity);
    }
}
