import { describe, expect, it, vi } from 'vitest';

import TunnelStateHandler from '../../src/main/tunnel-state';
import { TunnelState } from '../../src/shared/daemon-rpc-types';

const connected: TunnelState = { state: 'connected' } as TunnelState;
const connecting: TunnelState = { state: 'connecting' } as TunnelState;
const disconnected: TunnelState = { state: 'disconnected' } as TunnelState;
const disconnecting: TunnelState = { state: 'disconnecting' } as TunnelState;
const error: TunnelState = { state: 'error' } as TunnelState;

describe('Tunnel state', () => {
  it('Should allow all updates', () => {
    const stateUpdateSpy = vi.fn();

    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.handleNewTunnelState(disconnecting);
    tunnelStateHandler.handleNewTunnelState(connecting);
    tunnelStateHandler.handleNewTunnelState(error);
    tunnelStateHandler.handleNewTunnelState(disconnected);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(4);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(1, 'disconnecting');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(2, 'connecting');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(3, 'error');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(4, 'disconnected');
    expect(tunnelStateHandler.tunnelState.state).toEqual('disconnected');
  });

  it('Should accept any real state while expecting a predicted state', () => {
    const stateUpdateSpy = vi.fn();
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.handleNewTunnelState(error);
    tunnelStateHandler.expectNextTunnelState('disconnecting');
    tunnelStateHandler.handleNewTunnelState(disconnected);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(3);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(2, 'disconnecting');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(3, 'disconnected');
    expect(tunnelStateHandler.tunnelState.state).toEqual('disconnected');
  });

  it('Should time out and use last ignored state', () => {
    vi.useFakeTimers();
    const stateUpdateSpy = vi.fn();
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.handleNewTunnelState(disconnected);
    tunnelStateHandler.expectNextTunnelState('connecting');

    expect(stateUpdateSpy).toHaveBeenCalledTimes(2);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(1, 'disconnected');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(2, 'connecting');
    expect(tunnelStateHandler.tunnelState.state).toEqual('connecting');

    vi.advanceTimersByTime(3000);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(3);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(3, 'disconnected');
    expect(tunnelStateHandler.tunnelState.state).toEqual('disconnected');

    vi.useRealTimers();
  });
});
