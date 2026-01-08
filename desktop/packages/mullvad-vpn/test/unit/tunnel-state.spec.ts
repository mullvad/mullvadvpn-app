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

  it('Should ignore non-expected state update', () => {
    const stateUpdateSpy = vi.fn();
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnecting);
    tunnelStateHandler.handleNewTunnelState(connecting);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(2);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(1, 'connecting');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(2, 'connecting');
    expect(tunnelStateHandler.tunnelState.state).toEqual('connecting');
  });

  it('Should allow new states after expected state is reached', () => {
    const stateUpdateSpy = vi.fn();
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnected);
    tunnelStateHandler.handleNewTunnelState(connecting);
    tunnelStateHandler.handleNewTunnelState(connected);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(3);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(1, 'connecting');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(2, 'connecting');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(3, 'connected');
    expect(tunnelStateHandler.tunnelState.state).toEqual('connected');
  });

  it('Should allow error state update', () => {
    const stateUpdateSpy = vi.fn();
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnected);
    tunnelStateHandler.handleNewTunnelState(error);
    tunnelStateHandler.handleNewTunnelState(disconnected);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(3);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(1, 'connecting');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(2, 'error');
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(3, 'disconnected');
    expect(tunnelStateHandler.tunnelState.state).toEqual('disconnected');
  });

  it('Should time out and use last ignored state', () => {
    vi.useFakeTimers();
    const stateUpdateSpy = vi.fn();
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnected);
    tunnelStateHandler.handleNewTunnelState(connected);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(1);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(1, 'connecting');
    expect(tunnelStateHandler.tunnelState.state).toEqual('connecting');

    vi.advanceTimersByTime(3000);

    expect(stateUpdateSpy).toHaveBeenCalledTimes(2);
    expect(stateUpdateSpy).toHaveBeenNthCalledWith(2, 'connected');
    expect(tunnelStateHandler.tunnelState.state).toEqual('connected');

    vi.useRealTimers();
  });
});
