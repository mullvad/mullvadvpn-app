import { connectEnabled, disconnectEnabled, reconnectEnabled } from '../shared/connect-helper';
import { ILocation, TunnelState } from '../shared/daemon-rpc-types';
import { Scheduler } from '../shared/scheduler';

// Returns true if `to` is a valid completion of an optimistic `from` state.
function completesTransition(from: TunnelState['state'], to: TunnelState['state']): boolean {
  return (
    (from === 'connecting' && to === 'connected') ||
    (from === 'disconnecting' && to === 'disconnected') ||
    to === 'error'
  );
}

export interface TunnelStateProvider {
  getTunnelState(): TunnelState;
}

export interface TunnelStateHandlerDelegate {
  handleTunnelStateUpdate(tunnelState: TunnelState): void;
}

export default class TunnelStateHandler {
  // The current tunnel state
  private tunnelStateValue: TunnelState = { state: 'disconnected', lockedDown: false };
  // When pressing connect/disconnect/reconnect the app assumes what the next state will be before
  // it get's the new state from the daemon. The latest state from the daemon is saved as fallback
  // if the assumed state isn't reached.
  private tunnelStateFallback?: TunnelState;
  // Scheduler for discarding the assumed next state.
  private tunnelStateFallbackScheduler = new Scheduler();

  private lastKnownDisconnectedLocation: Partial<ILocation> | undefined;

  public constructor(private delegate: TunnelStateHandlerDelegate) {}

  public get tunnelState() {
    return this.tunnelStateValue;
  }

  public resetFallback() {
    this.tunnelStateFallbackScheduler.cancel();
    this.tunnelStateFallback = undefined;
  }

  // This function sets a new tunnel state as an assumed next state and saves the current state as
  // fallback. The fallback is used if the assumed next state isn't reached.
  // The timeout is needed to move out of the "predicted" state in case an actual state is never
  // emitted.
  public expectNextTunnelState(state: 'connecting' | 'disconnecting') {
    this.tunnelStateFallback = this.tunnelState;

    this.setTunnelState(
      state === 'disconnecting'
        ? { state, details: 'nothing' as const, location: this.lastKnownDisconnectedLocation }
        : { state, featureIndicators: undefined },
    );

    this.tunnelStateFallbackScheduler.schedule(() => {
      if (this.tunnelStateFallback) {
        this.setTunnelState(this.tunnelStateFallback);
        this.tunnelStateFallback = undefined;
      }
    }, 3000);
  }

  public handleNewTunnelState(newState: TunnelState) {
    // If there's a fallback state set then the app is in an assumed next state and need to check
    // if it's now reached or if the current state should be ignored and set as the fallback state.
    if (this.tunnelStateFallback) {
      this.tunnelStateFallbackScheduler.cancel();
      this.tunnelStateFallback = undefined;
    }

    if (newState.state === 'disconnecting' && newState.details === 'reconnect') {
      // The reconnecting state should appear the same as the connecting state.
      this.setTunnelState({ state: 'connecting', featureIndicators: undefined });
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
