import { expect, spy } from 'chai';
import { it, describe } from 'mocha';
import sinon from 'sinon';
import TunnelStateHandler from '../../src/main/tunnel-state';
import { TunnelState } from '../../src/shared/daemon-rpc-types';

const connected: TunnelState = { state: 'connected' } as TunnelState;
const connecting: TunnelState = { state: 'connecting' } as TunnelState;
const disconnected: TunnelState = { state: 'disconnected' } as TunnelState;
const disconnecting: TunnelState = { state: 'disconnecting' } as TunnelState;
const error: TunnelState = { state: 'error' } as TunnelState;

describe('Tunnel state', () => {
  it('Should allow all updates', () => {
    const stateUpdateSpy = spy();
    // @ts-ignore
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.handleNewTunnelState(disconnecting);
    tunnelStateHandler.handleNewTunnelState(connecting);
    tunnelStateHandler.handleNewTunnelState(error);
    tunnelStateHandler.handleNewTunnelState(disconnected);

    expect(stateUpdateSpy).to.have.been.called.exactly(4);
    expect(stateUpdateSpy).on.nth(1).to.have.been.called.with.exactly('disconnecting');
    expect(stateUpdateSpy).on.nth(2).to.have.been.called.with.exactly('connecting');
    expect(stateUpdateSpy).on.nth(3).to.have.been.called.with.exactly('error');
    expect(stateUpdateSpy).on.nth(4).to.have.been.called.with.exactly('disconnected');
    expect(tunnelStateHandler.tunnelState.state).to.equal('disconnected');
  });

  it('Should ignore non-expected state update', () => {
    const stateUpdateSpy = spy();
    // @ts-ignore
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnecting);
    tunnelStateHandler.handleNewTunnelState(connecting);

    expect(stateUpdateSpy).to.have.been.called.exactly(2);
    expect(stateUpdateSpy).on.nth(1).to.have.been.called.with.exactly('connecting');
    expect(stateUpdateSpy).on.nth(2).to.have.been.called.with.exactly('connecting');
    expect(tunnelStateHandler.tunnelState.state).to.equal('connecting');
  });

  it('Should allow new states after expected state is reached', () => {
    const stateUpdateSpy = spy();
    // @ts-ignore
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnected);
    tunnelStateHandler.handleNewTunnelState(connecting);
    tunnelStateHandler.handleNewTunnelState(connected);

    expect(stateUpdateSpy).to.have.been.called.exactly(3);
    expect(stateUpdateSpy).on.nth(1).to.have.been.called.with.exactly('connecting');
    expect(stateUpdateSpy).on.nth(2).to.have.been.called.with.exactly('connecting');
    expect(stateUpdateSpy).on.nth(3).to.have.been.called.with.exactly('connected');
    expect(tunnelStateHandler.tunnelState.state).to.equal('connected');
  });

  it('Should allow error state update', () => {
    const stateUpdateSpy = spy();
    // @ts-ignore
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnected);
    tunnelStateHandler.handleNewTunnelState(error);
    tunnelStateHandler.handleNewTunnelState(disconnected);

    expect(stateUpdateSpy).to.have.been.called.exactly(3);
    expect(stateUpdateSpy).on.nth(1).to.have.been.called.with.exactly('connecting');
    expect(stateUpdateSpy).on.nth(2).to.have.been.called.with.exactly('error');
    expect(stateUpdateSpy).on.nth(3).to.have.been.called.with.exactly('disconnected');
    expect(tunnelStateHandler.tunnelState.state).to.equal('disconnected');
  });

  it('Should time out and use last ignored state', () => {
    const clock = sinon.useFakeTimers({ shouldAdvanceTime: true });
    const stateUpdateSpy = spy();
    // @ts-ignore
    const handleTunnelStateUpdate = (tunnelState: TunnelState) => stateUpdateSpy(tunnelState.state);
    const tunnelStateHandler = new TunnelStateHandler({ handleTunnelStateUpdate });

    tunnelStateHandler.expectNextTunnelState('connecting');
    tunnelStateHandler.handleNewTunnelState(disconnected);
    tunnelStateHandler.handleNewTunnelState(connected);

    expect(stateUpdateSpy).to.have.been.called.exactly(1);
    expect(stateUpdateSpy).on.nth(1).to.have.been.called.with.exactly('connecting');
    expect(tunnelStateHandler.tunnelState.state).to.equal('connecting');

    clock.tick(3000);

    expect(stateUpdateSpy).to.have.been.called.exactly(2);
    expect(stateUpdateSpy).on.nth(2).to.have.been.called.with.exactly('connected');
    expect(tunnelStateHandler.tunnelState.state).to.equal('connected');
  });
});

