import { expect, spy } from 'chai';
import { it, describe, beforeEach } from 'mocha';
import History from '../../src/renderer/lib/history';
import { RoutePath } from '../../src/renderer/lib/routes';

const BASE_PATH = RoutePath.launch;
const FIRST_PATH = RoutePath.main;
const SECOND_PATH = RoutePath.settings;
const THIRD_PATH = RoutePath.vpnSettings;
const FOURTH_PATH = RoutePath.userInterfaceSettings;
const FIFTH_PATH = RoutePath.splitTunneling;

describe('History', () => {
  let history: History;

  beforeEach(() => {
    history = new History(BASE_PATH);
    history.push(FIRST_PATH);
    history.push(SECOND_PATH);
    history.push(THIRD_PATH);
    history.push(FOURTH_PATH);
  });

  it('should start at the correct location', () => {
    const history2 = new History(BASE_PATH);

    expect(history2.location.pathname).to.equal(BASE_PATH);
    expect(history2.length).to.equal(1);
    expect(history.location.pathname).to.equal(FOURTH_PATH);
    expect(history.length).to.equal(5);
  });

  it('should pop', () => {
    history.pop();
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(4);
  });

  it('should fail to pop', () => {
    history.pop();
    history.pop();
    history.pop();
    history.pop();

    expect(history.location.pathname).to.equal(BASE_PATH);
    expect(history.length).to.equal(1);

    history.pop();

    expect(history.location.pathname).to.equal(BASE_PATH);
    expect(history.length).to.equal(1);
  });

  it('should push', () => {
    history.push(FIFTH_PATH);
    expect(history.location.pathname).to.equal(FIFTH_PATH);
    expect(history.length).to.equal(6);
  });

  it('should go backward to base path', () => {
    history.pop(true);
    expect(history.location.pathname).to.equal(BASE_PATH);
    expect(history.length).to.equal(1);
  });

  it('should reset entries with path', () => {
    history.reset(THIRD_PATH);
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(1);
  });

  it('should add a listener', () => {
    const listenerA = spy();
    history.listen(listenerA);
    history.pop();
    history.push(FIFTH_PATH);

    const listenerB = spy();
    history.listen(listenerB);
    history.pop(true);
    history.push(FIRST_PATH);

    expect(listenerA).to.have.been.called.exactly(4);
    expect(listenerB).to.have.been.called.exactly(2);
  });

  it('should remove a listener', () => {
    const listenerA = spy();
    const removeListenerA = history.listen(listenerA);
    history.pop();
    history.push(FIFTH_PATH);

    const listenerB = spy();
    history.listen(listenerB);
    history.pop(true);

    removeListenerA();
    history.push(FIRST_PATH);
    history.reset(SECOND_PATH);

    expect(listenerA).to.have.been.called.exactly(3);
    expect(listenerB).to.have.been.called.exactly(3);
  });
});
