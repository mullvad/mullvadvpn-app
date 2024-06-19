import { connectEnabled, disconnectEnabled, reconnectEnabled } from '../shared/connect-helper';
import { ErrorStateCause, ILocation, TunnelState } from '../shared/daemon-rpc-types';
import { Scheduler } from '../shared/scheduler';

export interface TunnelStateProvider {
  getTunnelState(): TunnelState;
}

export interface TunnelStateHandlerDelegate {
  handleTunnelStateUpdate(tunnelState: TunnelState): void;
}

export default class TunnelStateHandler {
  // The current tunnel state
  private tunnelStateValue: TunnelState = { state: 'disconnected' };
  // When pressing connect/disconnect/reconnect the app assumes what the next state will be before
  // it get's the new state from the daemon. The latest state from the daemon is saved as fallback
  // if the assumed state isn't reached.
  private tunnelStateFallback?: TunnelState;
  // Scheduler for discarding the assumed next state.
  private tunnelStateFallbackScheduler = new Scheduler();

  private receivedFullDiskAccessError = false;

  private lastKnownDisconnectedLocation: Partial<ILocation> | undefined;

  public constructor(private delegate: TunnelStateHandlerDelegate) {}

  public get hasReceivedFullDiskAccessError() {
    return this.receivedFullDiskAccessError;
  }
  public get tunnelState() {
    return this.tunnelStateValue;
  }

  public resetFallback() {
    this.tunnelStateFallbackScheduler.cancel();
    this.tunnelStateFallback = undefined;
  }

  // This function sets a new tunnel state as an assumed next state and saves the current state as
  // fallback. The fallback is used if the assumed next state isn't reached.
  public expectNextTunnelState(state: 'connecting' | 'disconnecting') {
    this.tunnelStateFallback = this.tunnelState;

    this.setTunnelState(
      state === 'disconnecting'
        ? { state, details: 'nothing' as const, location: this.lastKnownDisconnectedLocation }
        : { state },
    );

    this.tunnelStateFallbackScheduler.schedule(() => {
      if (this.tunnelStateFallback) {
        this.setTunnelState(this.tunnelStateFallback);
        this.tunnelStateFallback = undefined;
      }
    }, 3000);
  }

  public handleNewTunnelState(newState: TunnelState) {
    if (newState.state === 'error') {
      if (newState.details.cause === ErrorStateCause.needFullDiskPermissions) {
        this.receivedFullDiskAccessError = true;
      }
    }

    // If there's a fallback state set then the app is in an assumed next state and need to check
    // if it's now reached or if the current state should be ignored and set as the fallback state.
    if (this.tunnelStateFallback) {
      if (this.tunnelState.state === newState.state || newState.state === 'error') {
        this.tunnelStateFallbackScheduler.cancel();
        this.tunnelStateFallback = undefined;
      } else {
        this.tunnelStateFallback = newState;
        return;
      }
    }

    if (newState.state === 'disconnecting' && newState.details === 'reconnect') {
      // When reconnecting there's no need of showing the disconnecting state. This switches to the
      // connecting state immediately.
      this.expectNextTunnelState('connecting');
      this.tunnelStateFallback = newState;
    } else {
      if (newState.state === 'disconnected' && newState.location !== undefined) {
        this.lastKnownDisconnectedLocation = newState.location;
      }

      if (
        newState.state === 'disconnecting' ||
        (newState.state === 'disconnected' && newState.location === undefined)
      ) {
        newState.location = this.lastKnownDisconnectedLocation;
      }

      this.setTunnelState(newState);
    }
  }

  public allowConnect(connectToDaemon: boolean, isLoggedIn: boolean) {
    return connectEnabled(connectToDaemon, isLoggedIn, this.tunnelState.state);
  }

  public allowReconnect(connectToDaemon: boolean, isLoggedIn: boolean) {
    return reconnectEnabled(connectToDaemon, isLoggedIn, this.tunnelState.state);
  }

  public allowDisconnect(connectToDaemon: boolean) {
    return disconnectEnabled(connectToDaemon, this.tunnelState.state);
  }

  private setTunnelState(newState: TunnelState) {
    this.tunnelStateValue = newState;
    this.delegate.handleTunnelStateUpdate(newState);
  }
}
