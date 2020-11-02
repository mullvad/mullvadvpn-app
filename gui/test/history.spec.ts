import { expect, spy } from 'chai';
import { it, describe, beforeEach } from 'mocha';
import History from '../src/renderer/lib/history';

const BASE_PATH = '/';
const FIRST_PATH = '/first-path';
const SECOND_PATH = '/second-path';
const THIRD_PATH = '/third-path';
const FOURTH_PATH = '/fourth-path';
const FIFTH_PATH = '/fifth-path';
const SIXTH_PATH = '/sixth-path';

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

  it('should go back', () => {
    history.goBack();
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(5);
  });

  it('should go back three entries', () => {
    history.go(-3);
    expect(history.location.pathname).to.equal(FIRST_PATH);
    expect(history.length).to.equal(5);
  });

  it('should go forward', () => {
    history.go(-3);
    history.goForward();
    expect(history.location.pathname).to.equal(SECOND_PATH);
    expect(history.length).to.equal(5);
  });

  it('should go forward two entries', () => {
    history.go(-3);
    history.go(2);
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(5);
  });

  it('should fail to go forward', () => {
    history.goForward();
    expect(history.location.pathname).to.equal(FOURTH_PATH);
    expect(history.length).to.equal(5);
  });

  it('should push', () => {
    history.push(FIFTH_PATH);
    history.goBack();
    expect(history.location.pathname).to.equal(FOURTH_PATH);
    expect(history.length).to.equal(6);
  });

  it('should replace', () => {
    history.replace(FIFTH_PATH);
    history.goBack();
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(5);
  });

  it('should fail to go backwards further than base path', () => {
    history.go(-5);
    expect(history.location.pathname).to.equal(FOURTH_PATH);
    expect(history.length).to.equal(5);
  });

  it('should go backward to base path', () => {
    history.reset();
    expect(history.location.pathname).to.equal(BASE_PATH);
    expect(history.length).to.equal(5);
  });

  it('should reset entries with path', () => {
    history.resetWith(THIRD_PATH);
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(1);

    history.goBack();
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(1);

    history.goForward();
    expect(history.location.pathname).to.equal(THIRD_PATH);
    expect(history.length).to.equal(1);
  });

  it('should fail to go forward after navigating', () => {
    history.goBack();
    history.push(FIFTH_PATH);
    history.goForward();
    expect(history.location.pathname).to.equal(FIFTH_PATH);

    history.goBack();
    history.replace(SIXTH_PATH);
    history.goForward();
    expect(history.location.pathname).to.equal(FIFTH_PATH);
  });

  it('should add a listener', () => {
    const listenerA = spy();
    history.listen(listenerA);
    history.goBack();
    history.goForward();

    const listenerB = spy();
    history.listen(listenerB);
    history.reset();
    history.push(FIRST_PATH);
    history.replace(SECOND_PATH);

    expect(listenerA).to.have.been.called.exactly(5);
    expect(listenerB).to.have.been.called.exactly(3);
  });

  it('should remove a listener', () => {
    const listenerA = spy();
    const removeListenerA = history.listen(listenerA);
    history.goBack();
    history.goForward();

    const listenerB = spy();
    history.listen(listenerB);
    history.reset();

    removeListenerA();
    history.push(FIRST_PATH);
    history.replace(SECOND_PATH);

    expect(listenerA).to.have.been.called.exactly(3);
    expect(listenerB).to.have.been.called.exactly(3);
  });

  it('should only remove listener once', () => {
    const listenerA = spy();
    const removeListenerA = history.listen(listenerA);
    history.goBack();

    const listenerB = spy();
    history.listen(listenerB);
    history.goForward();

    removeListenerA();
    removeListenerA();

    history.reset();

    expect(listenerA).to.have.been.called.exactly(2);
    expect(listenerB).to.have.been.called.exactly(2);
  });
});
